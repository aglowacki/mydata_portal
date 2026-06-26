//! EPICS Channel Access monitoring: one task per PV.
//!
//! Each task connects to its PV, reads the current value once and pushes it to the Redis
//! sink (so the hash holds a fresh snapshot on startup without waiting for a change), then
//! subscribes to value updates and forwards every new value to the sink via an mpsc
//! channel. On disconnect or error it backs off and reconnects, so the task runs for the
//! lifetime of the process.
//!
//! Value type: after connecting, the channel's native field type is inspected via
//! `chan.field_type()` and its element count via `chan.element_count()`, and the
//! subscription is dispatched to the matching Rust type. All scalar EPICS field types are
//! supported (char/short/long → integers, float/double → floating point, enum, and
//! string). Waveform (array) PVs — any channel whose element count is greater than one —
//! are supported too: they subscribe via the slice channel types
//! (`ValueChannel<[T]>::subscribe_vec`) and are stored as a `[a,b,c]` list (see
//! `format_array`) — except CHAR (`u8`) waveforms, which are decoded as an ASCII string
//! (the EPICS "long string" convention; see `FormatValue::format_slice` for `u8`).

use std::ffi::CString;
use std::time::Duration;

use epics_ca::types::{EpicsEnum, EpicsString, Field, FieldId};
use epics_ca::{Channel, Context};
use futures::{pin_mut, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn};

use crate::redis_sink::Update;

/// Monitor a single PV forever, reconnecting on failure.
///
/// `connect_timeout` bounds the wait for the initial connection; `reconnect_backoff` is
/// the pause between a disconnect and the next reconnect attempt.
pub async fn monitor_pv(
    ctx: Context,
    label: String,
    name: String,
    tx: mpsc::Sender<Update>,
    connect_timeout: Duration,
    reconnect_backoff: Duration,
) {
    let cname = match CString::new(name.as_str()) {
        Ok(c) => c,
        Err(e) => {
            warn!(pv = %name, error = %e, "PV name contains a NUL byte; skipping");
            return;
        }
    };

    loop {
        if let Err(reason) = subscribe_loop(&ctx, &cname, &label, &name, &tx, connect_timeout).await {
            // The Redis sink being gone is terminal: nothing left to write to.
            if tx.is_closed() {
                debug!(pv = %name, "sink closed; stopping monitor");
                return;
            }
            warn!(pv = %name, %reason, backoff = ?reconnect_backoff, "PV monitor dropped; reconnecting");
            sleep(reconnect_backoff).await;
        } else {
            // subscribe_loop only returns Ok when the sink is gone.
            return;
        }
    }
}

/// Connect, subscribe, and forward updates until the channel disconnects or the sink
/// closes. Returns `Err(reason)` on a recoverable failure (so the caller reconnects),
/// or `Ok(())` when the Redis sink has gone away (terminal).
async fn subscribe_loop(
    ctx: &Context,
    cname: &CString,
    label: &str,
    name: &str,
    tx: &mpsc::Sender<Update>,
    connect_timeout: Duration,
) -> Result<(), String> {
    let mut chan = Channel::new(ctx, cname).map_err(|e| format!("Channel::new failed: {e:?}"))?;

    // Wait (bounded) for the channel to connect.
    match timeout(connect_timeout, chan.connected()).await {
        Ok(()) => info!(pv = %name, "connected"),
        Err(_) => return Err("connection timed out".into()),
    }

    // Inspect the channel's native field type and element count, then subscribe with the
    // matching Rust type. A count greater than one means a waveform (array) PV, which is
    // routed to `run_array_subscription`; everything else is a scalar handled by
    // `run_subscription`. Both funnel through the `FormatValue` trait below.
    let field_type = chan
        .field_type()
        .map_err(|e| format!("field_type query failed: {e:?}"))?;
    let count = chan
        .element_count()
        .map_err(|e| format!("element_count query failed: {e:?}"))?;

    if count > 1 {
        match field_type {
            FieldId::Char => run_array_subscription::<u8>(chan, label, name, tx).await,
            FieldId::Short => run_array_subscription::<i16>(chan, label, name, tx).await,
            FieldId::Long => run_array_subscription::<i32>(chan, label, name, tx).await,
            FieldId::Float => run_array_subscription::<f32>(chan, label, name, tx).await,
            FieldId::Double => run_array_subscription::<f64>(chan, label, name, tx).await,
            FieldId::Enum => run_array_subscription::<EpicsEnum>(chan, label, name, tx).await,
            FieldId::String => run_array_subscription::<EpicsString>(chan, label, name, tx).await,
        }
    } else {
        match field_type {
            FieldId::Char => run_subscription::<u8>(chan, label, name, tx).await,
            FieldId::Short => run_subscription::<i16>(chan, label, name, tx).await,
            FieldId::Long => run_subscription::<i32>(chan, label, name, tx).await,
            FieldId::Float => run_subscription::<f32>(chan, label, name, tx).await,
            FieldId::Double => run_subscription::<f64>(chan, label, name, tx).await,
            FieldId::Enum => run_subscription::<EpicsEnum>(chan, label, name, tx).await,
            FieldId::String => run_subscription::<EpicsString>(chan, label, name, tx).await,
        }
    }
}

/// Subscribe to a scalar channel of field type `T` and forward each update to the sink.
///
/// Returns `Err(reason)` on a recoverable failure (the caller reconnects) or `Ok(())` when
/// the sink has gone away (terminal).
async fn run_subscription<T>(
    chan: Channel,
    label: &str,
    name: &str,
    tx: &mpsc::Sender<Update>,
) -> Result<(), String>
where
    T: Field + FormatValue,
{
    let typed = chan.into_typed::<T>().map_err(|(e, _)| {
        format!(
            "into_typed::<{}> failed (PV is not a scalar of this type?): {e:?}",
            std::any::type_name::<T>()
        )
    })?;
    let mut value = typed.into_value();

    // Push the current value to Redis immediately on startup, before subscribing, so the
    // hash reflects a fresh snapshot without waiting for the PV to change. A failure here
    // is non-fatal: we fall through to the subscription, which delivers values too.
    match value.get().await {
        Ok(v) => {
            let update = Update {
                key: label.to_string(),
                value: v.format_value(),
            };
            debug!(pv = %name, key = %label, value = %update.value, "initial value");
            if tx.send(update).await.is_err() {
                // Receiver dropped: the Redis sink is gone. Terminal.
                return Ok(());
            }
        }
        Err(e) => warn!(pv = %name, error = ?e, "failed to read initial value"),
    }

    let sub = value.subscribe();
    pin_mut!(sub);

    while let Some(item) = sub.next().await {
        match item {
            Ok(v) => {
                let update = Update {
                    key: label.to_string(),
                    value: v.format_value(),
                };
                debug!(pv = %name, key = %label, value = %update.value, "update");
                if tx.send(update).await.is_err() {
                    // Receiver dropped: the Redis sink is gone. Terminal.
                    return Ok(());
                }
            }
            Err(e) => return Err(format!("subscription error: {e:?}")),
        }
    }

    // Stream ended without error => treat as a disconnect and let the caller reconnect.
    Err("subscription stream ended".into())
}

/// Subscribe to a waveform (array) channel of element type `T` and forward each update.
///
/// Mirrors [`run_subscription`] but uses the slice channel types: it converts to
/// `ValueChannel<[T]>` and drives `subscribe_vec`, which yields a fresh `Vec<T>` per
/// update. The vector is rendered via [`FormatValue::format_slice`] — a `[a,b,c]` list for
/// most types, or a decoded ASCII string for CHAR (`u8`) waveforms.
///
/// Returns `Err(reason)` on a recoverable failure (the caller reconnects) or `Ok(())` when
/// the sink has gone away (terminal).
async fn run_array_subscription<T>(
    chan: Channel,
    label: &str,
    name: &str,
    tx: &mpsc::Sender<Update>,
) -> Result<(), String>
where
    T: Field + FormatValue,
{
    let typed = chan.into_typed::<[T]>().map_err(|(e, _)| {
        format!(
            "into_typed::<[{}]> failed (PV is not an array of this type?): {e:?}",
            std::any::type_name::<T>()
        )
    })?;
    let mut value = typed.into_value();

    // Push the current value to Redis immediately on startup, before subscribing, so the
    // hash reflects a fresh snapshot without waiting for the PV to change. A failure here
    // is non-fatal: we fall through to the subscription, which delivers values too.
    match value.get_vec().await {
        Ok(v) => {
            let update = Update {
                key: label.to_string(),
                value: T::format_slice(&v),
            };
            debug!(pv = %name, key = %label, len = v.len(), value = %update.value, "initial value");
            if tx.send(update).await.is_err() {
                // Receiver dropped: the Redis sink is gone. Terminal.
                return Ok(());
            }
        }
        Err(e) => warn!(pv = %name, error = ?e, "failed to read initial value"),
    }

    let sub = value.subscribe_vec();
    pin_mut!(sub);

    while let Some(item) = sub.next().await {
        match item {
            Ok(v) => {
                let update = Update {
                    key: label.to_string(),
                    value: T::format_slice(&v),
                };
                debug!(pv = %name, key = %label, len = v.len(), value = %update.value, "update");
                if tx.send(update).await.is_err() {
                    // Receiver dropped: the Redis sink is gone. Terminal.
                    return Ok(());
                }
            }
            Err(e) => return Err(format!("subscription error: {e:?}")),
        }
    }

    // Stream ended without error => treat as a disconnect and let the caller reconnect.
    Err("subscription stream ended".into())
}

/// Format a freshly-received PV value into the string stored in Redis.
///
/// Implemented per scalar field type so each gets a sensible textual form. Easy to swap
/// for JSON (value + timestamp + status) later.
trait FormatValue {
    fn format_value(&self) -> String;

    /// Render a whole waveform (array) of these values into the string stored in Redis.
    ///
    /// Defaults to a `[a,b,c]` list (see [`format_array`]); element types override this
    /// when an array has a more natural textual form. CHAR (`u8`) waveforms override it to
    /// decode as an ASCII string, the standard EPICS "long string" convention.
    fn format_slice(values: &[Self]) -> String
    where
        Self: Sized,
    {
        format_array(values)
    }
}

/// Format a waveform value as a `[a,b,c]` list, reusing each element's [`FormatValue`].
///
/// The bracketed, comma-separated form is unambiguous and easy for consumers to parse.
/// Swap it for JSON alongside `format_value` if richer output is needed later.
fn format_array<T: FormatValue>(values: &[T]) -> String {
    let mut out = String::with_capacity(2 + values.len() * 4);
    out.push('[');
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&v.format_value());
    }
    out.push(']');
    out
}

/// Format a floating-point value: kept tidy for whole numbers, full precision otherwise.
fn format_float(v: f64) -> String {
    if v.fract() == 0.0 && v.is_finite() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl FormatValue for f64 {
    fn format_value(&self) -> String {
        format_float(*self)
    }
}

impl FormatValue for f32 {
    fn format_value(&self) -> String {
        format_float(*self as f64)
    }
}

impl FormatValue for u8 {
    fn format_value(&self) -> String {
        self.to_string()
    }

    /// CHAR waveforms carry text by EPICS convention, so decode the bytes as an ASCII
    /// string rather than a numeric list. A NUL (0) byte terminates the string (the usual
    /// C convention used to pad fixed-size buffers), and any remaining non-printable bytes
    /// are dropped so control characters can't corrupt the stored value.
    fn format_slice(values: &[Self]) -> String {
        values
            .iter()
            .take_while(|&&b| b != 0)
            .filter(|&&b| b.is_ascii_graphic() || b == b' ')
            .map(|&b| b as char)
            .collect()
    }
}

impl FormatValue for i16 {
    fn format_value(&self) -> String {
        self.to_string()
    }
}

impl FormatValue for i32 {
    fn format_value(&self) -> String {
        self.to_string()
    }
}

impl FormatValue for EpicsEnum {
    fn format_value(&self) -> String {
        // EpicsEnum is a newtype over the raw u16 index; store the numeric index. (The
        // enum's string labels live in the GR/CTRL metadata, which we don't fetch here.)
        self.0.to_string()
    }
}

impl FormatValue for EpicsString {
    fn format_value(&self) -> String {
        // EpicsString derefs to a CStr; render it lossily so invalid UTF-8 can't panic.
        self.to_string_lossy().into_owned()
    }
}
