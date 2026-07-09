//! XRF live-map stream: subscribe to a ZeroMQ PUB/SUB feed of per-pixel fitted
//! counts produced by XRF-Maps, decode the binary `Stream_Block` payload, and
//! push each pixel's element/value pairs into Redis for later display on a
//! webpage.
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

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context as _, Result};
use redis::Commands;
use serde::Serialize;
use tracing::{error, info, warn};

use crate::config::XrfStreamConfig;

/// How long the SUB socket blocks on a receive before we loop back to check the
/// shutdown flag. Milliseconds.
const RECV_TIMEOUT_MS: i32 = 500;

/// One decoded pixel of the live XRF map. Serialized to JSON and stored in Redis.
#[derive(Debug, Serialize)]
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

/// Spawn the XRF stream listener on its own OS thread.
///
/// `cfg` is the optional `xrf_stream` config section; when it is `None` the
/// feature is disabled and this returns `None` so callers can treat the stream
/// as optional.
pub fn spawn(
    cfg: Option<XrfStreamConfig>,
    redis_url: String,
    running: Arc<AtomicBool>,
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
    if cfg.real_bytes != 4 && cfg.real_bytes != 8 {
        warn!(
            real_bytes = cfg.real_bytes,
            "`xrf_stream.real_bytes` invalid (expected 4 or 8); defaulting to 4"
        );
        cfg.real_bytes = 4;
    }

    Some(std::thread::spawn(move || {
        if let Err(e) = run(&cfg, &redis_url, &running) {
            error!(error = %e, "XRF stream listener terminated with error");
        }
    }))
}

/// Connect the SUB socket and pump messages into Redis until `running` clears.
fn run(cfg: &XrfStreamConfig, redis_url: &str, running: &AtomicBool) -> Result<()> {
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
    info!(%address, topic = %topic, real_bytes, "XRF live-map stream connected");

    let client = redis::Client::open(redis_url.to_string())
        .with_context(|| format!("opening Redis at {redis_url}"))?;
    let mut redis_conn = client
        .get_connection()
        .with_context(|| format!("connecting to Redis at {redis_url}"))?;

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
            Some(pixel) => push_pixel(&mut redis_conn, &pixel),
            None => warn!(len = payload.len(), "failed to decode XRF stream message"),
        }
    }

    info!("XRF live-map stream shutting down");
    Ok(())
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

/// Push a decoded pixel to Redis: store it in a per-dataset hash (keyed by
/// "<row>_<col>") for full retrieval and publish it on a same-named channel for
/// live updates.
fn push_pixel(conn: &mut redis::Connection, pixel: &XrfPixel) {
    let json = match serde_json::to_string(pixel) {
        Ok(j) => j,
        Err(e) => {
            warn!(error = %e, "failed to serialize XRF pixel");
            return;
        }
    };
    let map_key = format!("{}{}", defines::KEY_XRF_LIVE_MAP, pixel.dataset);
    let field = format!("{}_{}", pixel.row, pixel.col);

    let set: redis::RedisResult<()> = conn.hset(&map_key, &field, &json);
    if let Err(e) = set {
        warn!(key = %map_key, field = %field, error = %e, "failed to store XRF pixel");
    }
    // Best-effort live notification; a missing subscriber is not an error.
    let _: redis::RedisResult<()> = conn.publish(&map_key, &json);
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

/// Decode a counts message into an [`XrfPixel`]. Returns `None` on truncation or
/// on the XRF-Maps end-of-scan sentinel block (detector == -1).
fn decode_counts(message: &[u8], real_bytes: usize) -> Option<XrfPixel> {
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
        info!(%dataset, "received XRF end-of-scan block");
        return None;
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

    Some(XrfPixel {
        detector,
        row,
        col,
        height,
        width,
        theta,
        dataset,
        dataset_dir,
        elements,
    })
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
        let pixel = decode_counts(&msg, 4).expect("should decode");
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
    fn end_block_returns_none() {
        let msg = encode(-1, 0, 0, 0, 0, 0.0, "scan_042", "/data", &[]);
        assert!(decode_counts(&msg, 4).is_none());
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
}
