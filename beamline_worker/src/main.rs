//! beamline_worker — process beamline commands from a Redis queue while mirroring a set
//! of EPICS PVs into Redis.
//!
//! Two concerns run concurrently:
//!   * A dedicated EPICS thread monitors the configured PVs and writes their values into a
//!     Redis hash. EPICS Channel Access contexts are thread-affine and the subscription
//!     streams are not `Send`, so this thread owns a current-thread Tokio runtime and runs
//!     everything inside a `LocalSet`.
//!   * The main thread runs the synchronous ZeroMQ/Redis poll loop: it drains beamline log
//!     messages, services the Redis command queue, and emits a periodic heartbeat.
//!
//! Both shut down cleanly on Ctrl-C via a shared `running` flag.

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result};
use clap::Parser;
use epics_ca::Context;
use tokio::sync::mpsc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod beamline_controls;
mod command_protocols;
mod config;
mod epics;
mod redis_sink;
mod xrf_stream;

use config::Config;

/// Bound on the EPICS update channel; backpressures monitors if Redis falls behind.
const UPDATE_CHANNEL_CAPACITY: usize = 1024;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the JSON configuration file.
    #[arg(short, long, default_value = "config.json")]
    config_filename: String,
}

fn main() -> Result<()> {
    // Logging: honor RUST_LOG, default to info.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let args = Args::parse();
    // Read and parse the config file.
    let config_content =
        fs::read_to_string(&args.config_filename).expect("Error loading config file.");
    let config: Config =
        serde_json::from_str(&config_content).expect("Error parsing config file.");
    info!(
        config = %args.config_filename,
        pvs = config.ioc_config.pvs.len(),
        beamline_id = %config.beamline_id,
        "loaded configuration"
    );

    // Shared shutdown flag, flipped by the Ctrl-C handler and observed by both loops.
    let running = Arc::new(AtomicBool::new(true));

    // Ctrl-C handler: a tiny runtime on its own thread awaits the signal, then requests
    // shutdown. Kept off the main and EPICS threads so neither has to poll for the signal.
    {
        let running = running.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(rt) => rt,
                Err(e) => {
                    warn!(error = %e, "failed to build Ctrl-C runtime; shutdown signal disabled");
                    return;
                }
            };
            rt.block_on(async {
                if let Err(e) = tokio::signal::ctrl_c().await {
                    warn!(error = %e, "failed to listen for Ctrl-C");
                }
            });
            info!("Ctrl-C received; shutting down");
            running.store(false, Ordering::SeqCst);
        });
    }

    // EPICS monitor thread: owns its own current-thread runtime + LocalSet because CA
    // contexts are thread-affine and the subscription streams are not Send.
    let epics_handle = {
        let cfg = config.clone();
        let running = running.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("building EPICS Tokio runtime")?;
            let local = tokio::task::LocalSet::new();
            local.block_on(&rt, run_epics(cfg, running))
        })
    };

    // XRF live-map stream thread: subscribes to a ZeroMQ PUB/SUB feed (configured
    // via the `xrf_stream` config section) and mirrors decoded per-pixel counts
    // into Redis. Optional: returns None when the section is absent.
    let xrf_handle = xrf_stream::spawn(
        config.xrf_stream.clone(),
        config.redis_config.conn_str.clone(),
        running.clone(),
    );

    // Main thread: synchronous ZeroMQ + Redis command/log poll loop.
    if let Err(e) = run_command_loop(&config, &running) {
        warn!(error = %e, "command loop terminated with error");
        running.store(false, Ordering::SeqCst);
    }

    // Wait for the EPICS thread to flush and exit, but only briefly: its CA subscriptions
    // are torn down by dropping the runtime, which runs epics_ca's C destructors
    // (ca_clear_channel / ca_context_destroy). If that native teardown stalls, a plain
    // join() would hang the process forever, so we bound the wait and force-exit instead.
    let shutdown_deadline = Instant::now() + Duration::from_secs(5);
    while !epics_handle.is_finished() {
        if Instant::now() >= shutdown_deadline {
            warn!("EPICS monitor did not exit in time; forcing shutdown");
            std::process::exit(0);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    match epics_handle.join() {
        Ok(Ok(())) => {}
        Ok(Err(e)) => warn!(error = %e, "EPICS monitor terminated with error"),
        Err(_) => warn!("EPICS monitor thread panicked"),
    }

    // The XRF stream thread polls `running` on a bounded recv timeout, so this
    // join returns promptly once shutdown was requested.
    if let Some(handle) = xrf_handle {
        if handle.join().is_err() {
            warn!("XRF stream thread panicked");
        }
    }

    info!("stopped");
    Ok(())
}

/// Run the EPICS monitoring tasks until shutdown is requested.
///
/// Spawns one `monitor_pv` task per configured PV plus the Redis writer task, then waits
/// for the shared `running` flag to clear before tearing everything down.
async fn run_epics(cfg: Config, running: Arc<AtomicBool>) -> Result<()> {
    // The EPICS Channel Access context, shared by all monitor tasks.
    let ctx = Context::new().map_err(|e| anyhow::anyhow!("creating EPICS CA context: {e:?}"))?;

    let (tx, rx) = mpsc::channel(UPDATE_CHANNEL_CAPACITY);

    // Redis hash that all monitored PV values are written into, keyed per beamline.
    let hash_key = format!("{}{}", defines::KEY_IOC_MONITOR, cfg.beamline_id);

    // Start the Redis writer first so updates have somewhere to go.
    let writer = redis_sink::spawn(
        cfg.redis_config.conn_str.clone(),
        hash_key,
        rx,
    );

    let connect_timeout = Duration::from_secs(cfg.ioc_config.connect_timeout_secs);
    let reconnect_backoff = Duration::from_secs(cfg.ioc_config.reconnect_backoff_secs);

    // One monitor task per PV.
    for entry in &cfg.ioc_config.pvs {
        let ctx = ctx.clone();
        let tx = tx.clone();
        let label = entry.label.clone();
        let pv = entry.pv.clone();
        tokio::task::spawn_local(epics::monitor_pv(
            ctx,
            label,
            pv,
            tx,
            connect_timeout,
            reconnect_backoff,
        ));
    }
    // Drop our own sender so the writer can observe channel closure once all monitors end.
    drop(tx);

    info!(pvs = cfg.ioc_config.pvs.len(), "EPICS monitoring started");

    // Run until shutdown is requested.
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    info!("EPICS monitoring shutting down");

    // Dropping the runtime/LocalSet cancels the monitor tasks. Give the writer a brief
    // moment to flush; it will exit once its channel is closed.
    writer.abort();
    let _ = writer.await;
    Ok(())
}

/// Run the synchronous ZeroMQ/Redis command loop until shutdown is requested.
///
/// Mirrors the original beamline worker: drain log messages, service the command queue,
/// and emit a heartbeat every ten seconds.
fn run_command_loop(config: &Config, running: &AtomicBool) -> Result<()> {
    // ZeroMQ context and control clients.
    let context = zmq::Context::new();
    println!("Connected to Redis: {}", config.redis_config.conn_str);
    let mut client_map = beamline_controls::ClientMap::init(config, &context);

    let redis_client = redis::Client::open(config.redis_config.conn_str.clone())
        .context("Failed to connect to Redis")?;
    let mut redis_conn = redis_client
        .get_connection()
        .context("Failed to get Redis connection")?;

    println!(
        "Setting up redis command queue waiting {}",
        client_map.redis_key_cmd_queue_waiting
    );
    println!(
        "Setting up redis command queue processing {}",
        client_map.redis_key_cmd_queue_processing
    );
    println!(
        "Setting up redis command queue done {}",
        client_map.redis_key_cmd_queue_done
    );

    println!("Updating Redis with available scans");
    client_map.update_available_scans(&mut redis_conn);

    let mut heartbeat_start = Instant::now();
    let ten_seconds_dur = Duration::new(10, 0);

    println!("Polling for messages...");
    while running.load(Ordering::SeqCst) {
        // poll and publish logs
        client_map.poll_logs(&mut redis_conn);
        // poll redis command queue
        client_map.poll_cmd_queue(&mut redis_conn);
        // check if we should set heartbeat
        if heartbeat_start.elapsed() > ten_seconds_dur {
            client_map.send_heartbeat(&mut redis_conn);
            heartbeat_start = Instant::now();
        }
    }

    Ok(())
}
