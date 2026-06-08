/*
use std::process::{Command};
use std::fs::{OpenOptions};
use std::io::{Result};
use chrono::{Utc};
use std::fs;
use zmq;
use std::io::{self, Write};
//use std::thread;
//use std::time::Duration;

fn atoi(s: &str) -> u64 {
    s.parse().unwrap()
}

mod cmd_app;
//mod analysis_job;

fn run(app: &cmd_app::CmdApp) -> Result<()> 
{
    let now_str = Utc::now().format("%Y_%m_%d__%H_%M_%S").to_string();
    let stdout_name = format!("prog_{}.log",now_str);
    let stderr_name = format!("prog_{}.err",now_str);

    // Create files for stdout and stderr
    let stdout_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(stdout_name)?;
        
    let stderr_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(stderr_name)?;
    //println!{"{} {}", &app.path, &app.exe};
    // Define the command to execute the remote application
    let mut cmd = Command::new(&app.exe);
    cmd.current_dir(&app.path);
    cmd.stdout(stdout_file);
    cmd.stderr(stderr_file);

    // Redirect stdout and stderr to our custom files
    let mut child = cmd.spawn()?;
    
    // Wait for the process to finish
    let status = child.wait()?;
    
    // Write exit code to stderr
    println!("Process exited with code: {}", status.code().unwrap_or(-1));
    
    Ok(())
}

fn load_app_config(filename: &str) -> Result<cmd_app::CmdApp> 
{
    //println!{"{}",filename};
    let contents = fs::read_to_string(filename)
    .expect("Should have been able to read the file");
    //println!{"{}",contents};
    // don't unwrap like this in the real world! Errors will result in panic!
    let app_file: cmd_app::CmdApp = serde_yaml::from_str::<cmd_app::CmdApp>(&contents).unwrap();

    //println!("{:#?}", app_file);
    Ok(app_file)
}

fn main() 
{
    let app = load_app_config("../backend/runner_apps/xrf_maps.yml").unwrap();

    let context = zmq::Context::new();

    // socket to receive messages on
    let receiver = context.socket(zmq::PULL).unwrap();
    assert!(receiver.connect("tcp://localhost:5557").is_ok());

    //  Socket to send messages to
    let sender = context.socket(zmq::PUSH).unwrap();
    assert!(sender.connect("tcp://localhost:5558").is_ok());

    let controller = context.socket(zmq::SUB).unwrap();
    controller
        .connect("tcp://localhost:5559")
        .expect("failed connecting controller");
    controller.set_subscribe(b"").expect("failed subscribing");

    loop 
    {
        let mut items = [
            receiver.as_poll_item(zmq::POLLIN),
            controller.as_poll_item(zmq::POLLIN),
        ];
        zmq::poll(&mut items, -1).expect("failed polling");
        if items[0].is_readable() 
        {
            let string = receiver.recv_string(0).unwrap().unwrap();

            // Show progress
            print!(".");
            let _ = io::stdout().flush();

            // Do the work
            let res = run(&app);
            res.expect("Failed to run");
            //thread::sleep(Duration::from_millis(atoi(&string)));

            // Send results to sink
            sender.send("", 0).unwrap();
        }
        if items[1].is_readable() 
        {
            break;
        }
    }
}
*/

//-=-------------=------------=-------------=------------------=-------------

//
//
// RUST_LOG=debug cargo run -- --config config.json
//

/*
[package]
name = "redis_job_worker"
version = "0.2.0"
edition = "2021"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
redis = { version = "0.25", features = ["tokio-comp"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

*/

/*
{
  "job_id": "job-001",
  "worker_id": "worker-1234-1716735000000",
  "started_at": "2026-05-26T12:00:00Z",
  "heartbeat_at": "2026-05-26T12:00:20Z",
  "attempt": 0,
  "raw_job": "{\"job_id\":\"job-001\",\"attempt\":0,\"args\":{\"--input\":\"data/in.txt\"}}"
}

*/

//
/*
{
  "redis_url": "redis://127.0.0.1/",
  "queues": {
    "queued": "job_queue",
    "processing": "processing_queue",
    "finished": "finished_queue",
    "dead_letter": "dead_letter_queue"
  },
  "command": {
    "program": "/usr/bin/python3",
    "allowed_args": [
      "--input",
      "--output",
      "--mode",
      "--verbose"
    ],
    "working_dir": null,
    "stdout_dir": "logs/stdout",
    "stderr_dir": "logs/stderr"
  },
  "worker": {
    "max_retries": 3,
    "poll_timeout_secs": 5,
    "heartbeat_interval_secs": 10
  },
  "recovery": {
    "processing_set_key": "processing_jobs",
    "job_key_prefix": "jobmeta:",
    "stale_after_secs": 60,
    "scan_interval_secs": 15
  }
}

*/

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;
use tokio::signal;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about = "Async Redis-backed job worker with crash recovery")]
struct Cli {
    /// Path to JSON config file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct AdvertisingConfig {
    channel: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    redis_url: String,
    queues: QueueConfig,
    command: CommandConfig,
    worker: WorkerConfig,
    recovery: RecoveryConfig,
    advertising: AdvertisingConfig,
}

#[derive(Debug, Deserialize)]
struct QueueConfig {
    queued: String,
    processing: String,
    finished: String,
    dead_letter: String,
}

#[derive(Debug, Deserialize)]
struct CommandConfig {
    version: String,
    program: String,
    allowed_args: Vec<String>,
    working_dir: Option<String>,
    stdout_dir: String,
    stderr_dir: String,
}

#[derive(Debug, Deserialize)]
struct WorkerConfig {
    max_retries: u32,
    poll_timeout_secs: usize,
    heartbeat_interval_secs: u64,
}

#[derive(Debug, Deserialize)]
struct RecoveryConfig {
    processing_set_key: String,
    job_key_prefix: String,
    stale_after_secs: i64,
    scan_interval_secs: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Job {
    job_id: String,
    #[serde(default)]
    attempt: u32,
    args: HashMap<String, Option<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FinishedJobRecord {
    job_id: String,
    status: String,
    attempt: u32,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    exit_code: Option<i32>,
    stdout_file: String,
    stderr_file: String,
    error: Option<String>,
    original_job: Job,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeadLetterRecord {
    job_id: String,
    status: String,
    attempt: u32,
    failed_at: DateTime<Utc>,
    error: String,
    stdout_file: String,
    stderr_file: String,
    original_job: Job,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessingJobMeta {
    job_id: String,
    worker_id: String,
    started_at: DateTime<Utc>,
    heartbeat_at: DateTime<Utc>,
    attempt: u32,
    raw_job: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandAdvertisement {
    version: String,
    program: String,
    args: Vec<String>,
    advertised_at: DateTime<Utc>,
}

enum ProcessOutcome {
    Processed,
    NoJob,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let config = Arc::new(load_config(&cli.config).await?);
    ensure_dirs(&config).await?;

    let worker_id = format!("worker-{}-{}", std::process::id(), Utc::now().timestamp_millis());
    info!(worker_id = %worker_id, "worker starting");

    let redis_user = env::var("REDIS_USER").unwrap_or("default".to_string());
    let redis_pass = env::var("REDIS_PASS").expect("REDIS_PASS must be set");

    let client = redis::Client::open(config.redis_url.as_str())
        .with_context(|| format!("failed to create redis client for {}", config.redis_url))?;

    let mut work_conn = client
        .get_multiplexed_tokio_connection()
        .await
        .context("failed to connect to redis for work loop")?;

    // AUTH with username and password (Redis 6+ ACL)
    let auth_result: redis::RedisResult<String> = redis::cmd("AUTH")
        .arg(&redis_user)
        .arg(&redis_pass)
        .query_async(&mut work_conn)
        .await;

    match auth_result {
        Ok(response) => println!("Auth response: {}", response),
        Err(err) => error!(error = %err, "Authentication failed: {}",err),
    }

    advertise_command(&mut work_conn, &config).await?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let recovery_handle = tokio::spawn(recovery_loop(
        client.clone(),
        Arc::clone(&config),
        worker_id.clone(),
        shutdown_rx.clone(),
    ));

    let mut shutdown_handle = tokio::spawn(async move {
        if let Err(err) = shutdown_signal().await {
            error!(error = %err, "shutdown listener failed");
        }
    });

    loop {
        tokio::select! {
            _ = &mut shutdown_handle => {
                info!("shutdown signal received, stopping worker loop");
                break;
            }
            result = process_one_job(&config, &worker_id, &mut work_conn) => {
                match result {
                    Ok(ProcessOutcome::Processed) => {}
                    Ok(ProcessOutcome::NoJob) => {}
                    Err(err) => {
                        error!(error = %err, "error processing job");
                    }
                }
            }
        }
    }

    let _ = shutdown_tx.send(true);

    if let Err(err) = recovery_handle.await {
        error!(error = %err, "recovery task join failed");
    }

    info!("worker stopped");
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with_target(true)
        .init();
}

async fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config: Config = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse config JSON from {}", path.display()))?;
    Ok(config)
}

async fn ensure_dirs(config: &Config) -> Result<()> {
    fs::create_dir_all(&config.command.stdout_dir)
        .await
        .with_context(|| format!("failed to create stdout dir {}", config.command.stdout_dir))?;
    fs::create_dir_all(&config.command.stderr_dir)
        .await
        .with_context(|| format!("failed to create stderr dir {}", config.command.stderr_dir))?;
    Ok(())
}

async fn advertise_command(
    conn: &mut redis::aio::MultiplexedConnection,
    config: &Config,
) -> Result<()> {
    
    let payload = CommandAdvertisement {
        version: config.command.version.clone(),
        program: config.command.program.clone(),
        args: config.command.allowed_args.clone(),
        advertised_at: Utc::now(),
    };

    let payload_json = serde_json::to_string(&payload)
        .context("failed to serialize command advertisement")?;

    let subscribers: i64 = redis::cmd("PUBLISH")
        .arg(&config.advertising.channel)
        .arg(&payload_json)
        .query_async(conn)
        .await
        .with_context(|| {
            format!(
                "failed to publish command advertisement to channel {}",
                config.advertising.channel
            )
        })?;

    info!(
        channel = %config.advertising.channel,
        subscribers = subscribers,
        "published command advertisement"
    );

    Ok(())
}


async fn process_one_job(
    config: &Config,
    worker_id: &str,
    conn: &mut redis::aio::MultiplexedConnection,
) -> Result<ProcessOutcome> {
    let raw_job: Option<String> = brpoplpush_with_timeout(
        conn,
        &config.queues.queued,
        &config.queues.processing,
        config.worker.poll_timeout_secs,
    )
    .await
    .context("failed BRPOPLPUSH from queued to processing")?;

    let raw_job = match raw_job {
        Some(job) => job,
        None => return Ok(ProcessOutcome::NoJob),
    };

    let job: Job = serde_json::from_str(&raw_job)
        .with_context(|| format!("failed to parse job JSON: {}", raw_job))?;

    let started_at = Utc::now();

    register_processing_job(config, conn, &job, worker_id, started_at, &raw_job).await?;

    info!(
        job_id = %job.job_id,
        attempt = job.attempt,
        worker_id = %worker_id,
        "job received"
    );

    let heartbeat_client = redis::Client::open(config.redis_url.as_str())
        .with_context(|| format!("failed to create heartbeat redis client for {}", config.redis_url))?;

    let (heartbeat_stop_tx, heartbeat_stop_rx) = watch::channel(false);

    let heartbeat_job_id = job.job_id.clone();
    let heartbeat_config = config.recovery.job_key_prefix.clone();
    let heartbeat_interval = config.worker.heartbeat_interval_secs;

    let heartbeat_handle = tokio::spawn(async move {
        if let Err(err) = heartbeat_loop(
            heartbeat_client,
            heartbeat_config,
            heartbeat_job_id,
            heartbeat_interval,
            heartbeat_stop_rx,
        )
        .await
        {
            error!(error = %err, "heartbeat loop failed");
        }
    });

    let result = run_job(config, &job, started_at).await;

    let _ = heartbeat_stop_tx.send(true);
    let _ = heartbeat_handle.await;

    match result {
        Ok(record) => {
            info!(
                job_id = %record.job_id,
                attempt = record.attempt,
                exit_code = ?record.exit_code,
                "job finished successfully"
            );
            finalize_success(config, conn, &raw_job, &job.job_id, &record).await?;
        }
        Err(err) => {
            warn!(
                job_id = %job.job_id,
                attempt = job.attempt,
                error = %err,
                "job execution failed"
            );
            handle_failure(config, conn, &raw_job, &job, err).await?;
        }
    }

    Ok(ProcessOutcome::Processed)
}

async fn brpoplpush_with_timeout(
    conn: &mut redis::aio::MultiplexedConnection,
    source: &str,
    destination: &str,
    timeout_secs: usize,
) -> Result<Option<String>> {
    let result: Option<String> = redis::cmd("BRPOPLPUSH")
        .arg(source)
        .arg(destination)
        .arg(timeout_secs)
        .query_async(conn)
        .await?;
    Ok(result)
}

async fn register_processing_job(
    config: &Config,
    conn: &mut redis::aio::MultiplexedConnection,
    job: &Job,
    worker_id: &str,
    started_at: DateTime<Utc>,
    raw_job: &str,
) -> Result<()> {
    let meta = ProcessingJobMeta {
        job_id: job.job_id.clone(),
        worker_id: worker_id.to_string(),
        started_at,
        heartbeat_at: started_at,
        attempt: job.attempt,
        raw_job: raw_job.to_string(),
    };

    let meta_json = serde_json::to_string(&meta).context("failed to serialize processing metadata")?;
    let meta_key = processing_meta_key(config, &job.job_id);

    let _: () = redis::pipe()
        .atomic()
        .cmd("SADD")
        .arg(&config.recovery.processing_set_key)
        .arg(&job.job_id)
        .cmd("SET")
        .arg(&meta_key)
        .arg(meta_json)
        .query_async(conn)
        .await
        .context("failed to register processing job metadata")?;

    Ok(())
}

async fn heartbeat_loop(
    client: redis::Client,
    job_key_prefix: String,
    job_id: String,
    heartbeat_interval_secs: u64,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let mut conn = client
        .get_multiplexed_tokio_connection()
        .await
        .context("failed to connect redis for heartbeat loop")?;

    let redis_user = env::var("REDIS_USER").unwrap_or("default".to_string());
    let redis_pass = env::var("REDIS_PASS").expect("REDIS_PASS must be set");

    // AUTH with username and password (Redis 6+ ACL)
    let auth_result: redis::RedisResult<String> = redis::cmd("AUTH")
        .arg(&redis_user)
        .arg(&redis_pass)
        .query_async(&mut conn)
        .await;

    match auth_result {
        Ok(response) => println!("Auth response: {}", response),
        Err(err) => error!(error = %err, "Authentication failed: {}",err),
    }

    let meta_key = format!("{}{}", job_key_prefix, job_id);

    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    break;
                }
                if changed.is_err() {
                    break;
                }
            }
            _ = sleep(Duration::from_secs(heartbeat_interval_secs)) => {
                let meta_json: Option<String> = conn.get(&meta_key).await
                    .with_context(|| format!("failed to fetch processing metadata for {}", meta_key))?;

                let Some(meta_json) = meta_json else {
                    break;
                };

                let mut meta: ProcessingJobMeta = serde_json::from_str(&meta_json)
                    .with_context(|| format!("failed to parse processing metadata for {}", meta_key))?;

                meta.heartbeat_at = Utc::now();

                let updated = serde_json::to_string(&meta)
                    .context("failed to serialize updated heartbeat metadata")?;

                let _: () = conn.set(&meta_key, updated).await
                    .with_context(|| format!("failed to update heartbeat for {}", meta_key))?;
            }
        }
    }

    Ok(())
}

async fn run_job(
    config: &Config,
    job: &Job,
    started_at: DateTime<Utc>,
) -> Result<FinishedJobRecord> {
    let args = build_command_args(&config.command, &job.args)?;

    let stdout_path = stdout_file_path(config, job);
    let stderr_path = stderr_file_path(config, job);

    let stdout_file = std::fs::File::create(&stdout_path)
        .with_context(|| format!("failed to create stdout file {}", stdout_path.display()))?;
    let stderr_file = std::fs::File::create(&stderr_path)
        .with_context(|| format!("failed to create stderr file {}", stderr_path.display()))?;

    info!(
        job_id = %job.job_id,
        attempt = job.attempt,
        program = %config.command.program,
        args = ?args,
        "starting process"
    );

    let mut command = Command::new(&config.command.program);
    command.args(&args);
    command.stdout(Stdio::from(stdout_file));
    command.stderr(Stdio::from(stderr_file));

    if let Some(dir) = &config.command.working_dir {
        command.current_dir(dir);
    }

    let status = command
        .status()
        .await
        .with_context(|| format!("failed to execute program {}", config.command.program))?;

    let finished_at = Utc::now();

    let record = FinishedJobRecord {
        job_id: job.job_id.clone(),
        status: if status.success() {
            "finished".to_string()
        } else {
            "failed".to_string()
        },
        attempt: job.attempt,
        started_at,
        finished_at,
        exit_code: status.code(),
        stdout_file: stdout_path.display().to_string(),
        stderr_file: stderr_path.display().to_string(),
        error: if status.success() {
            None
        } else {
            Some(format!("process exited with status {:?}", status.code()))
        },
        original_job: job.clone(),
    };

    if status.success() {
        Ok(record)
    } else {
        Err(anyhow!(
            "process exited unsuccessfully with status {:?}",
            record.exit_code
        ))
    }
}

fn build_command_args(
    command_config: &CommandConfig,
    requested_args: &HashMap<String, Option<String>>,
) -> Result<Vec<String>> {
    let allowed: HashSet<&str> = command_config.allowed_args.iter().map(String::as_str).collect();

    let mut args = Vec::new();

    for (arg, value) in requested_args {
        if !allowed.contains(arg.as_str()) {
            return Err(anyhow!("argument '{}' is not allowed by config", arg));
        }

        args.push(arg.clone());

        if let Some(v) = value {
            args.push(v.clone());
        }
    }

    Ok(args)
}

fn stdout_file_path(config: &Config, job: &Job) -> PathBuf {
    Path::new(&config.command.stdout_dir)
        .join(format!("{}.attempt-{}.stdout.log", job.job_id, job.attempt))
}

fn stderr_file_path(config: &Config, job: &Job) -> PathBuf {
    Path::new(&config.command.stderr_dir)
        .join(format!("{}.attempt-{}.stderr.log", job.job_id, job.attempt))
}

fn processing_meta_key(config: &Config, job_id: &str) -> String {
    format!("{}{}", config.recovery.job_key_prefix, job_id)
}

async fn finalize_success(
    config: &Config,
    conn: &mut redis::aio::MultiplexedConnection,
    raw_job: &str,
    job_id: &str,
    finished_record: &FinishedJobRecord,
) -> Result<()> {
    let finished_json = serde_json::to_string(finished_record)
        .context("failed to serialize finished job record")?;
    let meta_key = processing_meta_key(config, job_id);

    let _: () = redis::pipe()
        .atomic()
        .cmd("LREM")
        .arg(&config.queues.processing)
        .arg(1)
        .arg(raw_job)
        .cmd("SREM")
        .arg(&config.recovery.processing_set_key)
        .arg(job_id)
        .cmd("DEL")
        .arg(meta_key)
        .cmd("LPUSH")
        .arg(&config.queues.finished)
        .arg(finished_json)
        .query_async(conn)
        .await
        .context("failed to finalize successful job in redis")?;

    Ok(())
}

async fn handle_failure(
    config: &Config,
    conn: &mut redis::aio::MultiplexedConnection,
    raw_job: &str,
    job: &Job,
    err: anyhow::Error,
) -> Result<()> {
    let stdout_path = stdout_file_path(config, job);
    let stderr_path = stderr_file_path(config, job);

    let next_attempt = job.attempt + 1;
    let meta_key = processing_meta_key(config, &job.job_id);

    if next_attempt < config.worker.max_retries {
        let mut retried_job = job.clone();
        retried_job.attempt = next_attempt;

        let retried_json =
            serde_json::to_string(&retried_job).context("failed to serialize retried job")?;

        warn!(
            job_id = %job.job_id,
            current_attempt = job.attempt,
            next_attempt = next_attempt,
            max_retries = config.worker.max_retries,
            "requeueing failed job"
        );

        let _: () = redis::pipe()
            .atomic()
            .cmd("LREM")
            .arg(&config.queues.processing)
            .arg(1)
            .arg(raw_job)
            .cmd("SREM")
            .arg(&config.recovery.processing_set_key)
            .arg(&job.job_id)
            .cmd("DEL")
            .arg(meta_key)
            .cmd("LPUSH")
            .arg(&config.queues.queued)
            .arg(retried_json)
            .query_async(conn)
            .await
            .context("failed to requeue failed job")?;
    } else {
        let dlq_record = DeadLetterRecord {
            job_id: job.job_id.clone(),
            status: "dead_letter".to_string(),
            attempt: job.attempt,
            failed_at: Utc::now(),
            error: format!("{err:#}"),
            stdout_file: stdout_path.display().to_string(),
            stderr_file: stderr_path.display().to_string(),
            original_job: job.clone(),
        };

        let dlq_json = serde_json::to_string(&dlq_record)
            .context("failed to serialize dead-letter record")?;

        error!(
            job_id = %job.job_id,
            attempt = job.attempt,
            max_retries = config.worker.max_retries,
            "moving job to dead-letter queue"
        );

        let _: () = redis::pipe()
            .atomic()
            .cmd("LREM")
            .arg(&config.queues.processing)
            .arg(1)
            .arg(raw_job)
            .cmd("SREM")
            .arg(&config.recovery.processing_set_key)
            .arg(&job.job_id)
            .cmd("DEL")
            .arg(meta_key)
            .cmd("LPUSH")
            .arg(&config.queues.dead_letter)
            .arg(dlq_json)
            .query_async(conn)
            .await
            .context("failed to move job to dead-letter queue")?;
    }

    Ok(())
}

async fn recovery_loop(
    client: redis::Client,
    config: Arc<Config>,
    worker_id: String,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let mut conn = client
        .get_multiplexed_tokio_connection()
        .await
        .context("failed to connect redis for recovery loop")?;


    let redis_user = env::var("REDIS_USER").unwrap_or("default".to_string());
    let redis_pass = env::var("REDIS_PASS").expect("REDIS_PASS must be set");

    // AUTH with username and password (Redis 6+ ACL)
    let auth_result: redis::RedisResult<String> = redis::cmd("AUTH")
        .arg(&redis_user)
        .arg(&redis_pass)
        .query_async(&mut conn)
        .await;

    match auth_result {
        Ok(response) => println!("Auth response: {}", response),
        Err(err) => error!(error = %err, "Authentication failed: {}",err),
    }
    
    info!(
        worker_id = %worker_id,
        stale_after_secs = config.recovery.stale_after_secs,
        scan_interval_secs = config.recovery.scan_interval_secs,
        "recovery loop started"
    );

    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    info!("recovery loop shutting down");
                    break;
                }
                if changed.is_err() {
                    break;
                }
            }
            _ = sleep(Duration::from_secs(config.recovery.scan_interval_secs)) => {
                if let Err(err) = scan_and_recover_stale_jobs(&config, &mut conn).await {
                    error!(error = %err, "recovery scan failed");
                }
            }
        }
    }

    Ok(())
}

async fn scan_and_recover_stale_jobs(
    config: &Config,
    conn: &mut redis::aio::MultiplexedConnection,
) -> Result<()> {
    let job_ids: Vec<String> = conn
        .smembers(&config.recovery.processing_set_key)
        .await
        .context("failed to fetch processing job ids")?;

    let now = Utc::now();

    for job_id in job_ids {
        let meta_key = processing_meta_key(config, &job_id);

        let meta_json: Option<String> = conn
            .get(&meta_key)
            .await
            .with_context(|| format!("failed to get metadata for job {}", job_id))?;

        let Some(meta_json) = meta_json else {
            warn!(job_id = %job_id, "processing set contains job without metadata; cleaning up");
            let _: () = conn
                .srem(&config.recovery.processing_set_key, &job_id)
                .await
                .with_context(|| format!("failed to cleanup missing metadata job {}", job_id))?;
            continue;
        };

        let meta: ProcessingJobMeta = serde_json::from_str(&meta_json)
            .with_context(|| format!("failed to parse processing metadata for job {}", job_id))?;

        let age = now.signed_duration_since(meta.heartbeat_at).num_seconds();
        if age <= config.recovery.stale_after_secs {
            continue;
        }

        warn!(
            job_id = %meta.job_id,
            worker_id = %meta.worker_id,
            heartbeat_at = %meta.heartbeat_at,
            stale_age_secs = age,
            "stale processing job detected"
        );

        let job: Job = serde_json::from_str(&meta.raw_job)
            .with_context(|| format!("failed to parse raw job JSON for stale job {}", meta.job_id))?;

        recover_stale_job(config, conn, &meta, &job).await?;
    }

    Ok(())
}

async fn recover_stale_job(
    config: &Config,
    conn: &mut redis::aio::MultiplexedConnection,
    meta: &ProcessingJobMeta,
    job: &Job,
) -> Result<()> {
    let next_attempt = job.attempt + 1;
    let meta_key = processing_meta_key(config, &job.job_id);

    if next_attempt < config.worker.max_retries {
        let mut retried_job = job.clone();
        retried_job.attempt = next_attempt;

        let retried_json =
            serde_json::to_string(&retried_job).context("failed to serialize recovered retry job")?;

        warn!(
            job_id = %job.job_id,
            previous_attempt = job.attempt,
            next_attempt = next_attempt,
            "recovering stale job back to queued"
        );

        let _: () = redis::pipe()
            .atomic()
            .cmd("LREM")
            .arg(&config.queues.processing)
            .arg(1)
            .arg(&meta.raw_job)
            .cmd("SREM")
            .arg(&config.recovery.processing_set_key)
            .arg(&job.job_id)
            .cmd("DEL")
            .arg(meta_key)
            .cmd("LPUSH")
            .arg(&config.queues.queued)
            .arg(retried_json)
            .query_async(conn)
            .await
            .context("failed to recover stale job to queued")?;
    } else {
        let dlq_record = DeadLetterRecord {
            job_id: job.job_id.clone(),
            status: "dead_letter".to_string(),
            attempt: job.attempt,
            failed_at: Utc::now(),
            error: format!(
                "job became stale in processing after worker crash or lost heartbeat; last worker_id={}, last heartbeat_at={}",
                meta.worker_id, meta.heartbeat_at
            ),
            stdout_file: stdout_file_path(config, job).display().to_string(),
            stderr_file: stderr_file_path(config, job).display().to_string(),
            original_job: job.clone(),
        };

        let dlq_json = serde_json::to_string(&dlq_record)
            .context("failed to serialize stale dead-letter record")?;

        error!(
            job_id = %job.job_id,
            attempt = job.attempt,
            "stale job exceeded retries; moving to dead-letter queue"
        );

        let _: () = redis::pipe()
            .atomic()
            .cmd("LREM")
            .arg(&config.queues.processing)
            .arg(1)
            .arg(&meta.raw_job)
            .cmd("SREM")
            .arg(&config.recovery.processing_set_key)
            .arg(&job.job_id)
            .cmd("DEL")
            .arg(meta_key)
            .cmd("LPUSH")
            .arg(&config.queues.dead_letter)
            .arg(dlq_json)
            .query_async(conn)
            .await
            .context("failed to move stale job to dead-letter queue")?;
    }

    Ok(())
}

async fn shutdown_signal() -> Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm =
            signal(SignalKind::terminate()).context("failed to install SIGTERM handler")?;

        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("received Ctrl+C");
            }
            _ = sigterm.recv() => {
                info!("received SIGTERM");
            }
        }
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c()
            .await
            .context("failed to listen for Ctrl+C")?;
        info!("received Ctrl+C");
    }

    Ok(())
}
