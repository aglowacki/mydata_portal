//! XRF live-map stream: subscribe to a ZeroMQ PUB/SUB feed of per-pixel fitted
//! counts produced by XRF-Maps, decode the binary `Stream_Block` payload, and
//! persist each scan into a zarr dataset on disk. A lightweight event is
//! published to Redis at the end of every completed row so a webpage can refresh.
//!
//! ## Storage layout
//!
//! Each scan is written to a filesystem zarr store at
//! `<output_dir>/<dataset>.zarr` containing a root group and one 2-D `f64` array
//! per fitted element, named after the (sanitized) element. Every array has:
//!   * shape `[height, width]` (the scan's N rows x M cols, from the meta block)
//!   * chunk shape `[1, width]` -- one chunk per row, so a completed row is a
//!     single chunk write
//!   * fill value `NaN` (pixels that were never written read back as NaN)
//!
//! Rows are buffered in memory and flushed once complete (detected when a pixel
//! for a new row arrives, or on the end-of-scan sentinel). Flushing writes one
//! chunk per element and then publishes a small JSON notification to the Redis
//! channel `KEY_XRF_LIVE_MAP + dataset`.
//!
//! ## Wire format
//!
//! The binary wire format is produced by XRF-Maps' `Basic_Serializer`
//! (`io/net/basic_serializer.cpp`, `encode_counts`). It is written with raw
//! `memcpy` in native (x86 little-endian) byte order, so we decode as
//! little-endian here. Layout of a counts message:
//!
//! Meta:
//!   detector   : u32
//!   row        : size_t (8 bytes)
//!   col        : size_t (8 bytes)
//!   height     : size_t (8 bytes)   -- N (number of rows in the scan)
//!   width      : size_t (8 bytes)   -- M (number of cols in the scan)
//!   theta      : T_real (real_bytes)
//!   dataset    : null-terminated string
//!   dataset_dir: null-terminated string
//! Counts:
//!   proc_type_count : u32
//!   for each fitting routine:
//!     proc_type      : u32
//!     fit_block_size : u32
//!     for each element:
//!       name  : null-terminated string
//!       value : T_real (real_bytes)
//!
//! The size of `T_real` is not encoded in the stream (see the TODO in the C++
//! encoder), so it is configurable via the `real_bytes` config field and
//! defaults to 4 (float), which is what XRF-Maps streams by default.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context as _, Result};
use redis::Commands;
use serde::Serialize;
use tracing::{error, info, warn};
use zarrs::array::{data_type, Array, ArrayBuilder};
use zarrs::filesystem::FilesystemStore;
use zarrs::group::GroupBuilder;
use zarrs::storage::{ReadableWritableListableStorage, ReadableWritableListableStorageTraits};

use crate::config::XrfStreamConfig;

/// How long the SUB socket blocks on a receive before we loop back to check the
/// shutdown flag. Milliseconds.
const RECV_TIMEOUT_MS: i32 = 500;

/// One decoded pixel of the live XRF map.
#[derive(Debug)]
struct XrfPixel {
    detector: i32,
    row: u64,
    col: u64,
    height: u64,
    width: u64,
    theta: f64,
    dataset: String,
    dataset_dir: String,
    /// Element name -> fitted count value. BTreeMap keeps a stable ordering.
    elements: BTreeMap<String, f64>,
}

/// Result of decoding a stream message.
#[derive(Debug)]
enum Decoded {
    /// A normal per-pixel counts block.
    Pixel(XrfPixel),
    /// The XRF-Maps end-of-scan sentinel (detector == -1).
    EndOfScan { dataset: String },
}

/// The Redis notification published at the end of each completed row.
#[derive(Debug, Serialize)]
struct RowEvent<'a> {
    dataset: &'a str,
    row: u64,
    height: u64,
    width: u64,
}

/// Spawn the XRF stream listener on its own OS thread.
///
/// `cfg` is the optional `xrf_stream` config section; when it is `None` the
/// feature is disabled and this returns `None` so callers can treat the stream
/// as optional.
pub fn spawn(
    cfg: Option<XrfStreamConfig>,
    redis_url: String,
    running: Arc<AtomicBool>,
    debug: bool,
) -> Option<std::thread::JoinHandle<()>> {
    let mut cfg = match cfg {
        Some(cfg) => cfg,
        None => {
            info!("no `xrf_stream` config section; XRF live-map stream disabled");
            return None;
        }
    };
    if cfg.host.trim().is_empty() {
        warn!("`xrf_stream.host` is empty; XRF live-map stream disabled");
        return None;
    }
    if cfg.output_dir.trim().is_empty() {
        warn!("`xrf_stream.output_dir` is empty; XRF live-map stream disabled");
        return None;
    }
    if cfg.real_bytes != 4 && cfg.real_bytes != 8 {
        warn!(
            real_bytes = cfg.real_bytes,
            "`xrf_stream.real_bytes` invalid (expected 4 or 8); defaulting to 4"
        );
        cfg.real_bytes = 4;
    }

    Some(std::thread::spawn(move || {
        if let Err(e) = run(&cfg, &redis_url, &running, debug) {
            error!(error = %e, "XRF stream listener terminated with error");
        }
    }))
}

/// Connect the SUB socket and pump messages into per-dataset zarr stores until
/// `running` clears, publishing a Redis event at the end of each row.
fn run(cfg: &XrfStreamConfig, redis_url: &str, running: &AtomicBool, debug: bool) -> Result<()> {
    let topic = cfg.topic.as_str();
    let real_bytes = cfg.real_bytes;
    let ctx = zmq::Context::new();
    let subscriber = ctx
        .socket(zmq::SUB)
        .context("creating XRF stream SUB socket")?;
    // LINGER = 0 so context termination on shutdown never blocks on undelivered frames.
    subscriber.set_linger(0).context("setting SUB linger")?;
    subscriber
        .set_rcvtimeo(RECV_TIMEOUT_MS)
        .context("setting SUB recv timeout")?;

    let address = format!("tcp://{}:{}", cfg.host, cfg.port);
    subscriber
        .connect(&address)
        .with_context(|| format!("connecting XRF stream SUB to {address}"))?;
    subscriber
        .set_subscribe(topic.as_bytes())
        .context("subscribing to XRF stream topic")?;
    info!(%address, topic = %topic, real_bytes, output_dir = %cfg.output_dir, "XRF live-map stream connected");

    let client = redis::Client::open(redis_url.to_string())
        .with_context(|| format!("opening Redis at {redis_url}"))?;
    let mut redis_conn = client
        .get_connection()
        .with_context(|| format!("connecting to Redis at {redis_url}"))?;

    // One writer per in-flight dataset, keyed by dataset name.
    let mut writers: HashMap<String, DatasetWriter> = HashMap::new();

    while running.load(Ordering::SeqCst) {
        // Receive a (possibly multipart) message. With a PUB/SUB topic the topic
        // is typically the first frame and the binary payload the last.
        let parts = match subscriber.recv_multipart(0) {
            Ok(parts) => parts,
            Err(zmq::Error::EAGAIN) => continue, // recv timed out; re-check `running`
            Err(e) => {
                warn!(error = %e, "XRF stream receive failed");
                continue;
            }
        };

        let payload = match extract_payload(&parts, topic.as_bytes()) {
            Some(p) => p,
            None => continue,
        };

        match decode_counts(payload, real_bytes) {
            Some(Decoded::Pixel(pixel)) => {
                if debug {
                    debug_print_pixel(payload.len(), &pixel);
                }
                // A fresh scan announces itself with `col == 0` under a dataset name we
                // are not already writing. XRF-Maps does not reliably send an
                // end-of-scan sentinel, so treat the new scan's first pixel as the
                // signal to finalize (flush the last row + publish) any previously
                // in-flight scans and close out their zarr stores.
                if pixel.col == 0 && !writers.contains_key(&pixel.dataset) && !writers.is_empty() {
                    for (prev_dataset, mut writer) in writers.drain() {
                        if debug {
                            println!(
                                "[xrf-debug] new dataset {:?} at col 0; finalizing previous {:?}",
                                pixel.dataset, prev_dataset
                            );
                        }
                        writer.finish(&mut redis_conn);
                    }
                }
                if !writers.contains_key(&pixel.dataset) {
                    match DatasetWriter::new(&cfg.output_dir, &pixel, debug) {
                        Ok(w) => {
                            writers.insert(pixel.dataset.clone(), w);
                        }
                        Err(e) => {
                            warn!(dataset = %pixel.dataset, error = %e, "failed to create XRF zarr store");
                            continue;
                        }
                    }
                }
                if let Some(writer) = writers.get_mut(&pixel.dataset) {
                    writer.accumulate(&pixel, &mut redis_conn);
                }
            }
            Some(Decoded::EndOfScan { dataset }) => {
                if debug {
                    println!(
                        "[xrf-debug] end-of-scan sentinel: dataset={dataset:?} (payload {} bytes)",
                        payload.len()
                    );
                }
                info!(%dataset, "received XRF end-of-scan block");
                if let Some(mut writer) = writers.remove(&dataset) {
                    writer.finish(&mut redis_conn);
                }
            }
            None => warn!(len = payload.len(), "failed to decode XRF stream message"),
        }
    }

    info!("XRF live-map stream shutting down");
    Ok(())
}

/// Print a decoded pixel's meta and element counts to stdout, before it is folded
/// into the zarr store. Enabled by the `--debug` flag; intended for diagnosing
/// empty/metadata-only stores (e.g. when no element counts are being decoded).
fn debug_print_pixel(payload_len: usize, pixel: &XrfPixel) {
    println!(
        "[xrf-debug] pixel dataset={:?} det={} row={} col={} height={} width={} theta={} \
         elements={} (payload {} bytes) dir={:?}",
        pixel.dataset,
        pixel.detector,
        pixel.row,
        pixel.col,
        pixel.height,
        pixel.width,
        pixel.theta,
        pixel.elements.len(),
        payload_len,
        pixel.dataset_dir,
    );
    for (name, value) in &pixel.elements {
        println!("[xrf-debug]     {name} = {value}");
    }
}

/// A zarr store for a single scan plus the in-memory buffer for the row currently
/// being filled.
struct DatasetWriter {
    store: ReadableWritableListableStorage,
    dataset: String,
    height: u64,
    width: u64,
    /// Element name -> its 2-D `[height, width]` zarr array, created lazily as
    /// elements first appear in the stream.
    arrays: BTreeMap<String, Array<dyn ReadableWritableListableStorageTraits>>,
    /// Row index currently buffered, if any.
    cur_row: Option<u64>,
    /// Element name -> buffered values for the current row (length `width`, NaN-filled).
    buf: BTreeMap<String, Vec<f64>>,
    /// Emit `[xrf-debug]` write-path tracing (set from the `--debug` flag).
    debug: bool,
}

impl DatasetWriter {
    /// Create the on-disk store and root group for a scan from its first pixel.
    fn new(output_dir: &str, pixel: &XrfPixel, debug: bool) -> Result<Self> {
        let path = Path::new(output_dir).join(format!("{}.zarr", sanitize_name(&pixel.dataset)));
        let store: ReadableWritableListableStorage = Arc::new(
            FilesystemStore::new(&path)
                .with_context(|| format!("creating zarr store at {}", path.display()))?,
        );

        let mut attributes = serde_json::Map::new();
        attributes.insert("detector".into(), pixel.detector.into());
        attributes.insert("height".into(), pixel.height.into());
        attributes.insert("width".into(), pixel.width.into());
        attributes.insert("theta".into(), pixel.theta.into());
        attributes.insert("dataset".into(), pixel.dataset.clone().into());
        attributes.insert("dataset_dir".into(), pixel.dataset_dir.clone().into());

        GroupBuilder::new()
            .attributes(attributes)
            .build(store.clone(), "/")
            .context("building XRF zarr root group")?
            .store_metadata()
            .context("storing XRF zarr group metadata")?;

        info!(dataset = %pixel.dataset, path = %path.display(), height = pixel.height, width = pixel.width, "created XRF zarr store");

        Ok(Self {
            store,
            dataset: pixel.dataset.clone(),
            height: pixel.height,
            width: pixel.width,
            arrays: BTreeMap::new(),
            cur_row: None,
            buf: BTreeMap::new(),
            debug,
        })
    }

    /// Fold one pixel into the current row buffer.
    ///
    /// Flushing is driven by the column index, because XRF-Maps streams a scan
    /// column-by-column (`col` runs 0..width-1 then restarts at 0 for the next
    /// row):
    ///   * `col == 0` marks the start of a new row, so we flush whatever was
    ///     buffered for the previous row before beginning the new one.
    ///   * `col == width-1` is the end of a row, so we flush it immediately
    ///     rather than waiting for the next row's `col 0` -- this ensures the
    ///     final row of a scan lands on disk.
    fn accumulate(&mut self, pixel: &XrfPixel, redis_conn: &mut redis::Connection) {
        // Start of a new row: flush the previous one (written at the old `cur_row`).
        if pixel.col == 0 {
            self.flush_row(redis_conn);
            self.buf.clear();
        }
        self.cur_row = Some(pixel.row);

        if pixel.col >= self.width {
            warn!(dataset = %self.dataset, col = pixel.col, width = self.width, "XRF pixel column out of range; dropping");
            return;
        }
        let width = self.width as usize;
        for (name, value) in &pixel.elements {
            let row_buf = self
                .buf
                .entry(name.clone())
                .or_insert_with(|| vec![f64::NAN; width]);
            row_buf[pixel.col as usize] = *value;
        }

        // End of the row: flush now so the row is persisted without waiting for the
        // next row (important for the last row of the scan).
        if pixel.col + 1 >= self.width {
            self.flush_row(redis_conn);
            self.buf.clear();
        }
    }

    /// Write the buffered row (one chunk per element) and publish a Redis event.
    /// No-op when there is nothing buffered.
    fn flush_row(&mut self, redis_conn: &mut redis::Connection) {
        let row = match self.cur_row {
            Some(r) if !self.buf.is_empty() => r,
            _ => {
                if self.debug {
                    println!(
                        "[xrf-debug] flush_row: nothing to write (cur_row={:?}, buffered_elements={})",
                        self.cur_row,
                        self.buf.len()
                    );
                }
                return;
            }
        };

        // Take the buffer so we can mutate `self.arrays` without aliasing.
        let buf = std::mem::take(&mut self.buf);
        if self.debug {
            println!(
                "[xrf-debug] flush_row: writing row {} for {} element(s): {:?}",
                row,
                buf.len(),
                buf.keys().collect::<Vec<_>>()
            );
        }
        for (name, values) in &buf {
            let array = match self.get_or_create_array(name) {
                Some(a) => a,
                None => continue,
            };
            if let Err(e) = array.store_chunk(&[row, 0], values.as_slice()) {
                warn!(dataset = %self.dataset, element = %name, row, error = %e, "failed to write XRF row chunk");
            } else if self.debug {
                let non_nan = values.iter().filter(|v| !v.is_nan()).count();
                println!(
                    "[xrf-debug]     wrote {name} row {row}: {non_nan}/{} non-NaN values",
                    values.len()
                );
            }
        }

        let event = RowEvent {
            dataset: &self.dataset,
            row,
            height: self.height,
            width: self.width,
        };
        match serde_json::to_string(&event) {
            Ok(json) => {
                let channel = format!("{}{}", defines::KEY_XRF_LIVE_MAP, self.dataset);
                // Best-effort live notification; a missing subscriber is not an error.
                let _: redis::RedisResult<()> = redis_conn.publish(&channel, &json);
            }
            Err(e) => warn!(dataset = %self.dataset, error = %e, "failed to serialize XRF row event"),
        }
    }

    /// Flush any remaining buffered row at end-of-scan.
    fn finish(&mut self, redis_conn: &mut redis::Connection) {
        self.flush_row(redis_conn);
    }

    /// Get the zarr array for `name`, creating a `[height, width]` array (chunked
    /// one row per chunk, NaN fill) on first use. Returns `None` if creation fails.
    fn get_or_create_array(
        &mut self,
        name: &str,
    ) -> Option<&Array<dyn ReadableWritableListableStorageTraits>> {
        if !self.arrays.contains_key(name) {
            let node_path = format!("/{}", sanitize_name(name));
            let array = ArrayBuilder::new(
                vec![self.height, self.width],
                vec![1, self.width],
                data_type::float64(),
                f64::NAN,
            )
            .dimension_names(["row", "col"].into())
            .build(self.store.clone(), &node_path);

            let array = match array {
                Ok(a) => a,
                Err(e) => {
                    warn!(dataset = %self.dataset, element = %name, error = %e, "failed to build XRF element array");
                    return None;
                }
            };
            if let Err(e) = array.store_metadata() {
                warn!(dataset = %self.dataset, element = %name, error = %e, "failed to store XRF element array metadata");
                return None;
            }
            self.arrays.insert(name.to_string(), array);
        }
        self.arrays.get(name)
    }
}

/// Make an element/dataset name safe as a zarr node name: node names may not
/// contain `/` and must not be `.`/`..`. We keep ASCII alphanumerics and
/// `-`, `_`, `.`, mapping everything else to `_`.
fn sanitize_name(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.is_empty() || out == "." || out == ".." {
        out = format!("_{out}");
    }
    out
}

/// Pull the binary payload frame out of a received message, stripping the topic
/// prefix if the publisher packs topic + payload into a single frame.
fn extract_payload<'a>(parts: &'a [Vec<u8>], topic: &[u8]) -> Option<&'a [u8]> {
    match parts.len() {
        0 => None,
        1 => {
            let frame = parts[0].as_slice();
            // Single frame: strip a leading topic prefix if present.
            if !topic.is_empty() && frame.starts_with(topic) {
                Some(&frame[topic.len()..])
            } else {
                Some(frame)
            }
        }
        // Multipart: topic frame(s) first, binary payload last.
        _ => Some(parts[parts.len() - 1].as_slice()),
    }
}

/// A little-endian cursor over the message bytes.
struct Cursor<'a> {
    buf: &'a [u8],
    idx: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, idx: 0 }
    }

    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.idx)
    }

    fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.remaining() < n {
            return None;
        }
        let out = &self.buf[self.idx..self.idx + n];
        self.idx += n;
        Some(out)
    }

    fn read_u32(&mut self) -> Option<u32> {
        let b = self.read_bytes(4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|v| v as i32)
    }

    /// C++ `size_t`, 8 bytes on the 64-bit hosts this streams from.
    fn read_size_t(&mut self) -> Option<u64> {
        let b = self.read_bytes(8)?;
        Some(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// `T_real`, either 4-byte float or 8-byte double, returned widened to f64.
    fn read_real(&mut self, real_bytes: usize) -> Option<f64> {
        let b = self.read_bytes(real_bytes)?;
        match real_bytes {
            4 => Some(f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64),
            8 => Some(f64::from_le_bytes([
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            ])),
            _ => None,
        }
    }

    /// Read a null-terminated string (the null is consumed).
    fn read_cstr(&mut self) -> Option<String> {
        let start = self.idx;
        while self.idx < self.buf.len() && self.buf[self.idx] != 0 {
            self.idx += 1;
        }
        if self.idx >= self.buf.len() {
            return None; // no terminator found
        }
        let s = String::from_utf8_lossy(&self.buf[start..self.idx]).into_owned();
        self.idx += 1; // consume the null terminator
        Some(s)
    }
}

/// Decode a counts message into a [`Decoded`] value. Returns `None` only on
/// truncation/parse failure; the end-of-scan sentinel (detector == -1) is
/// returned as [`Decoded::EndOfScan`].
fn decode_counts(message: &[u8], real_bytes: usize) -> Option<Decoded> {
    let mut c = Cursor::new(message);

    // --- meta ---
    let detector = c.read_i32()?;
    let row = c.read_size_t()?;
    let col = c.read_size_t()?;
    let height = c.read_size_t()?;
    let width = c.read_size_t()?;
    let theta = c.read_real(real_bytes)?;
    let dataset = c.read_cstr()?;
    let dataset_dir = c.read_cstr()?;

    // XRF-Maps sends a sentinel "end block" with detector == -1 to mark the end
    // of a scan; there are no counts to decode for it.
    if detector == -1 {
        return Some(Decoded::EndOfScan { dataset });
    }

    // --- counts ---
    let mut elements: BTreeMap<String, f64> = BTreeMap::new();
    let proc_type_count = c.read_u32()?;
    for _ in 0..proc_type_count {
        let _proc_type = c.read_u32()?; // fitting routine id; merged into one map
        let fit_block_size = c.read_u32()?;
        for _ in 0..fit_block_size {
            let name = c.read_cstr()?;
            let value = c.read_real(real_bytes)?;
            // Multiple fitting routines may report the same element; keep the last.
            elements.insert(name, value);
        }
    }

    Some(Decoded::Pixel(XrfPixel {
        detector,
        row,
        col,
        height,
        width,
        theta,
        dataset,
        dataset_dir,
        elements,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a counts message the way `Basic_Serializer::encode_counts` would,
    /// for real_bytes = 4.
    fn encode(
        detector: i32,
        row: u64,
        col: u64,
        height: u64,
        width: u64,
        theta: f32,
        dataset: &str,
        dataset_dir: &str,
        routines: &[&[(&str, f32)]],
    ) -> Vec<u8> {
        let mut m = Vec::new();
        m.extend_from_slice(&(detector as u32).to_le_bytes());
        m.extend_from_slice(&row.to_le_bytes());
        m.extend_from_slice(&col.to_le_bytes());
        m.extend_from_slice(&height.to_le_bytes());
        m.extend_from_slice(&width.to_le_bytes());
        m.extend_from_slice(&theta.to_le_bytes());
        m.extend_from_slice(dataset.as_bytes());
        m.push(0);
        m.extend_from_slice(dataset_dir.as_bytes());
        m.push(0);
        m.extend_from_slice(&(routines.len() as u32).to_le_bytes());
        for (i, r) in routines.iter().enumerate() {
            m.extend_from_slice(&(i as u32).to_le_bytes()); // proc_type
            m.extend_from_slice(&(r.len() as u32).to_le_bytes());
            for (name, val) in r.iter() {
                m.extend_from_slice(name.as_bytes());
                m.push(0);
                m.extend_from_slice(&val.to_le_bytes());
            }
        }
        m
    }

    fn expect_pixel(decoded: Option<Decoded>) -> XrfPixel {
        match decoded {
            Some(Decoded::Pixel(p)) => p,
            other => panic!("expected a pixel, got {other:?}"),
        }
    }

    #[test]
    fn decodes_a_pixel() {
        let msg = encode(
            0,
            0,
            0,
            100,
            100,
            0.0,
            "scan_042",
            "/data",
            &[&[("S", 200.1), ("Fe", 423.03), ("Cu", 9000.2)]],
        );
        let pixel = expect_pixel(decode_counts(&msg, 4));
        assert_eq!(pixel.detector, 0);
        assert_eq!(pixel.height, 100);
        assert_eq!(pixel.width, 100);
        assert_eq!(pixel.dataset, "scan_042");
        assert_eq!(pixel.elements.len(), 3);
        assert!((pixel.elements["S"] - 200.1).abs() < 1e-3);
        assert!((pixel.elements["Fe"] - 423.03).abs() < 1e-3);
        assert!((pixel.elements["Cu"] - 9000.2).abs() < 1e-3);
    }

    #[test]
    fn end_block_returns_end_of_scan() {
        let msg = encode(-1, 0, 0, 0, 0, 0.0, "scan_042", "/data", &[]);
        match decode_counts(&msg, 4) {
            Some(Decoded::EndOfScan { dataset }) => assert_eq!(dataset, "scan_042"),
            other => panic!("expected end-of-scan, got {other:?}"),
        }
    }

    #[test]
    fn truncated_message_returns_none() {
        let msg = encode(0, 1, 2, 10, 10, 0.0, "d", "/data", &[&[("Fe", 1.0)]]);
        assert!(decode_counts(&msg[..msg.len() - 2], 4).is_none());
    }

    #[test]
    fn strips_single_frame_topic_prefix() {
        let parts = vec![b"XRF\x00\x01\x02".to_vec()];
        assert_eq!(extract_payload(&parts, b"XRF"), Some(&b"\x00\x01\x02"[..]));
    }

    #[test]
    fn multipart_takes_last_frame() {
        let parts = vec![b"XRF".to_vec(), b"\x01\x02".to_vec()];
        assert_eq!(extract_payload(&parts, b"XRF"), Some(&b"\x01\x02"[..]));
    }

    #[test]
    fn sanitize_name_maps_invalid_chars() {
        assert_eq!(sanitize_name("Fe"), "Fe");
        assert_eq!(sanitize_name("Fe_K"), "Fe_K");
        assert_eq!(sanitize_name("a/b c"), "a_b_c");
        assert_eq!(sanitize_name(""), "_");
        assert_eq!(sanitize_name(".."), "_..");
    }
}
