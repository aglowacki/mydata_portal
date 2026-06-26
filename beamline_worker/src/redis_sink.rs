//! Redis sink: a single task that drains PV updates and writes them into a hash.
//!
//! Centralizing all Redis I/O in one task (fed by an mpsc channel) decouples EPICS
//! polling from Redis, keeps all Redis error handling in one place, and avoids sharing
//! a connection across the per-PV monitor tasks.

use anyhow::{Context, Result};
use redis::AsyncCommands;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// A single PV value update destined for Redis.
#[derive(Debug, Clone)]
pub struct Update {
    /// The configured label; used as the hash field.
    pub key: String,
    /// The formatted value to store.
    pub value: String,
}

/// Run the Redis writer loop until the update channel is closed.
///
/// `hash_key` is the Redis hash that every PV value is written into as a field.
pub async fn run(redis_url: String, hash_key: String, mut rx: mpsc::Receiver<Update>) -> Result<()> {
    let client =
        redis::Client::open(redis_url.clone()).with_context(|| format!("opening {redis_url}"))?;
    // A multiplexed connection transparently reconnects, so we keep a single one for
    // the lifetime of the process.
    let mut conn = client
        .get_multiplexed_async_connection()
        .await
        .with_context(|| format!("connecting to Redis at {redis_url}"))?;
    info!(%redis_url, %hash_key, "connected to Redis");

    while let Some(update) = rx.recv().await {
        // HSET <hash_key> <label> <value>. Failures are logged but don't kill the task;
        // the multiplexed connection retries the underlying socket on its own.
        let result: redis::RedisResult<()> =
            conn.hset(&hash_key, &update.key, &update.value).await;
        match result {
            Ok(()) => {}
            Err(e) => warn!(key = %update.key, error = %e, "failed to write update to Redis"),
        }
    }

    info!("update channel closed; Redis writer shutting down");
    Ok(())
}

/// Spawn [`run`] as a local task, logging a fatal error if it returns one.
pub fn spawn(
    redis_url: String,
    hash_key: String,
    rx: mpsc::Receiver<Update>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_local(async move {
        if let Err(e) = run(redis_url, hash_key, rx).await {
            error!(error = %e, "Redis writer terminated");
        }
    })
}
