use zmq::PollItem;
use redis::{Commands};
use std::sync::Arc;
use crate::{beamline_controls, config};
use std::time::Duration;
use chrono::{DateTime, Utc};

use crate::{command_protocols};
use defines;

pub struct ControlClient
{
    cmd_address: String,
    log_address: String,
    log_topic: String,
    beamline_id: String,
    beamline_id_log: String,
    beamline_id_event: String,
    protocol: String,
    subscriber: zmq::Socket,
    cmd_channel: zmq::Socket,
}

impl ControlClient 
{
    pub fn new(config: &config::ControlClient, beamline_id: &str, context: &zmq::Context) -> Self
    {
        Self
        {
            cmd_address: format!("tcp://{}:60615", config.host.to_string()),
            log_address: format!("tcp://{}:60625", config.host.to_string()),
            log_topic: String::from(config.zmq_log_topic.clone()),
            protocol: String::from(config.protocol.clone()),
            beamline_id: String::from(beamline_id),
            beamline_id_log: format!("{}{}", defines::KEY_BEAMLINE_SCAN_LOGS, beamline_id),
            beamline_id_event: format!("{}{}", defines::KEY_BEAMLINE_EVENT, beamline_id),
            subscriber: context.socket(zmq::SUB).expect("Failed to create SUB socket"),
            cmd_channel: context.socket(zmq::REQ).expect("Failed to create CMD socket"),
        }
    }

    pub fn connect(&self)
    {
        // LINGER = 0 so closing the sockets / terminating the context on shutdown never
        // blocks waiting to flush undelivered messages. Without this, a REQ request that
        // was queued but never answered (e.g. a timed-out command, or an unreachable
        // BlueSky peer) makes zmq_ctx_term hang forever and the process never exits.
        self.subscriber.set_linger(0).expect("Failed to set SUB linger");
        self.cmd_channel.set_linger(0).expect("Failed to set CMD linger");
        println!("Connecting to ZeroMQ PUB at {}", self.log_address);
        self.subscriber.connect(&self.log_address).expect("Failed to connect to PUB socket");
        self.subscriber.set_subscribe(self.log_topic.as_bytes()).expect("Failed to subscribe");
        println!("Connecting to ZeroMQ CMD at {}", self.cmd_address);
        self.cmd_channel.connect(&self.cmd_address).expect("Failed to connect to CMD socket");
    }

    pub fn gen_poll_item(&self) -> PollItem<'_>
    {
        return self.subscriber.as_poll_item(zmq::POLLIN);
    }

}

pub struct ClientMap
{
    pub redis_key_cmd_queue_waiting: String,
    pub redis_key_cmd_queue_processing: String,
    pub redis_key_cmd_queue_done: String,
    pub redis_key_heartbeat: String,
    // Program-wide beamline id; commands are only handled if they target it.
    beamline_id: String,
    // The single control client this worker drives.
    client: Arc<ControlClient>,
    poll_list: Vec<PollItem<'static>>,
    // Last queueserver `plan_queue_uid` cached/published. Used to suppress
    // duplicate queue events when a log line only looks like a queue change.
    last_queue_uid: Option<String>,
    // Last queueserver `plan_history_uid` cached/published. Used to publish a
    // history event only when a scan actually finished (a new history entry).
    last_history_uid: Option<String>,
}

/// Heuristic test for whether a queueserver console log line reflects a change
/// to the scan queue or the running scan. Two kinds of change are detected:
///
///   * Queue content: an item added, removed, or reordered. Matched on "queue"
///     together with an action keyword.
///   * Scan lifecycle: a plan starting or finishing (which changes the running
///     item and empties/advances the queue). Matched on the specific phrases the
///     queueserver RE Manager prints around those transitions.
///
/// The phrasing varies by queueserver version, so matching is case-insensitive.
/// The lifecycle phrases are kept specific (not bare words like "complete") so
/// they don't fire on every per-reading progress line during a scan. False
/// positives are harmless anyway: the caller re-fetches the authoritative queue
/// and only publishes an event when its composite UID actually changed.
fn is_queue_change_message(msg: &str) -> bool
{
    let lower = msg.to_lowercase();

    // Queue content changes: "queue" plus an action verb.
    if lower.contains("queue")
    {
        const ACTIONS: [&str; 8] = ["add", "insert", "remov", "delet", "move", "moving", "reorder", "clear"];
        if ACTIONS.iter().any(|a| lower.contains(a))
        {
            return true;
        }
    }

    // Scan lifecycle transitions: a plan starting/finishing or the RE Manager
    // changing run state. These bracket scan execution and move items between
    // the queue and the running slot.
    const LIFECYCLE: [&str; 5] = [
        "starting the plan",            // a queued plan began running
        "plan exit status",             // a plan finished (completed/failed/stopped)
        "queue is empty",               // last plan finished, nothing left to run
        "changing the state of re manager", // idle <-> executing_queue transitions
        "no plans left in the queue",
    ];
    LIFECYCLE.iter().any(|p| lower.contains(p))
}

/// Build a composite change key for a `queue_get` reply: the `plan_queue_uid`
/// (which changes when items are added/removed/reordered) combined with the
/// currently running item's `item_uid` (which changes when a scan starts or
/// finishes). Comparing this composite — rather than `plan_queue_uid` alone —
/// means we also detect a running scan completing, which leaves the queue list
/// (and thus `plan_queue_uid`) unchanged but clears `running_item`.
/// Returns None when the reply is not valid JSON.
fn extract_queue_uid(reply: &str) -> Option<String>
{
    let value = serde_json::from_str::<serde_json::Value>(reply).ok()?;
    let queue_uid = value
        .get("plan_queue_uid")
        .and_then(|u| u.as_str())
        .unwrap_or("");
    // Empty when idle (running_item is an empty object with no item_uid).
    let running_uid = value
        .get("running_item")
        .and_then(|r| r.get("item_uid"))
        .and_then(|u| u.as_str())
        .unwrap_or("");
    Some(format!("{}|{}", queue_uid, running_uid))
}

/// Pull the `plan_history_uid` out of a `history_get` reply. It changes whenever a
/// scan finishes and a new entry is appended to the history, so we use it to
/// publish a history event only on real changes. Returns None when the reply is
/// not valid JSON or lacks the field.
fn extract_history_uid(reply: &str) -> Option<String>
{
    serde_json::from_str::<serde_json::Value>(reply)
        .ok()
        .and_then(|v| v.get("plan_history_uid").and_then(|u| u.as_str()).map(|s| s.to_string()))
}

fn process_protocol_command(cmd_client: &ControlClient, beamline_cmd: &mut command_protocols::BeamlineCommand)
{
    let gen_result = command_protocols::generate_cmd(&cmd_client.protocol, &beamline_cmd);
    match gen_result
    {
        Some(mut protocol_cmd)=>
        {
            beamline_cmd.proc_start_time = Some(Utc::now());
            let reply_result = protocol_cmd.execute(&cmd_client.cmd_channel);
            match reply_result
            {
                Ok(reply) => 
                {
                    beamline_cmd.status = "Completed".to_string();
                    beamline_cmd.reply = Some(reply);
                }
                Err(e) => beamline_cmd.status = e.message().to_string(),
            }
            beamline_cmd.proc_end_time = Some(Utc::now());
        }
        None =>
        {
                println!("Failed to traslate command for protocol {}", cmd_client.protocol);
                beamline_cmd.set_unable_to_translate_protocol()
        }
    }  
}

impl ClientMap
{
    pub fn init(config: &config::Config, context: &zmq::Context ) -> Self
    {
        let bs_client = Arc::new(ControlClient::new(&config.control_client, &config.beamline_id, context));
        println!("New BlueSky Client: Cmd: {} , Log: {}", bs_client.cmd_address, bs_client.log_address);
        bs_client.connect();
        // SAFETY: We extend the lifetime to 'static because Arc ensures the data lives long enough.
        let poll_item: PollItem<'static> = unsafe { std::mem::transmute(bs_client.gen_poll_item()) };
        let poll_list: Vec<PollItem<'static>> = vec![poll_item];

        Self
        {
            redis_key_cmd_queue_waiting: format!("{}{}", defines::KEY_TASK_QUEUE_WAITING, config.beamline_id.to_string()),
            redis_key_cmd_queue_processing: format!("{}{}", defines::KEY_TASK_QUEUE_PROCESSING, config.beamline_id.to_string()),
            redis_key_cmd_queue_done: format!("{}{}", defines::KEY_TASK_QUEUE_DONE, config.beamline_id.to_string()),
            redis_key_heartbeat: format!("{}{}", defines::KEY_WORKER_HEARTBEAT, config.beamline_id.to_string()),
            beamline_id: config.beamline_id.clone(),
            client: bs_client,
            poll_list,
            last_queue_uid: None,
            last_history_uid: None,
        }
    }

    pub fn poll_logs(&mut self, redis_conn: &mut redis::Connection) -> bool
    {
        let mut got_data = false;
        let poll_timeout = 100; // Poll timeout in milliseconds
        // Poll the socket
        let poll_result = zmq::poll(&mut self.poll_list, poll_timeout);

        match poll_result
        {
            Ok(_) =>
            {
                if self.poll_list[0].is_readable()
                {
                    // Clone the Arc (cheap) rather than borrowing self, so we can call the
                    // &mut self queue-refresh method below once this scope is done.
                    let client = self.client.clone();
                    // Set when a received log line looks like a scan-queue change; handled
                    // after the borrow of `client` ends to avoid a borrow conflict.
                    let mut queue_changed = false;
                    match client.subscriber.recv_string(0)
                    {
                        Ok(Ok(message)) =>
                        {
                            println!("Received message: {}", message);
                            got_data = true;
                            if message != client.log_topic
                            {
                                // Push the message to the Redis list
                                let result: redis::RedisResult<()> = redis_conn.rpush(&(client.beamline_id_log), &message);
                                // Publish on the beamline event channel, prefixed with the
                                // controls-event header and ':::' delimiter.
                                let event_msg = format!("{}:::{}", defines::KEY_CONTROLS_EVENT, message);
                                let _: redis::RedisResult<()> = redis_conn.publish(&(client.beamline_id_event), &event_msg);
                                match result
                                {
                                    Ok(_) => println!("Message pushed to Redis list: {}", client.beamline_id_log),
                                    Err(err) => eprintln!("Failed to push message to Redis: {}", err),
                                }

                                // Console log lines arrive as JSON {time, msg}; inspect the
                                // inner `msg` text (falling back to the raw line) for
                                // queue-change activity.
                                let log_text = serde_json::from_str::<serde_json::Value>(&message)
                                    .ok()
                                    .and_then(|v| v.get("msg").and_then(|m| m.as_str()).map(|s| s.to_string()))
                                    .unwrap_or_else(|| message.clone());
                                if is_queue_change_message(&log_text)
                                {
                                    queue_changed = true;
                                }
                            }
                        }
                        Ok(Err(err)) => eprintln!("Failed to decode message: {}", String::from_utf8(err).expect("Failed to decode!")),
                        Err(err) => eprintln!("Failed to receive message: {}", err),
                    }
                    // `client` (Arc clone) is dropped here, releasing any implicit hold,
                    // so we can take &mut self to re-fetch and publish if needed.
                    drop(client);
                    if queue_changed
                    {
                        // Refresh the history too: a finished scan both changes the
                        // queue/running state and appends a history entry. The UID
                        // guard inside publishes a HISTORY_EVENT only when a scan
                        // actually finished, so add/remove churn costs one extra
                        // (unpublished) fetch but produces no spurious events.
                        self.refresh_history_if_changed(redis_conn);
                        self.refresh_queue_if_changed(redis_conn);
                    }
                }
            }
            Err(err) => {
                eprintln!("Polling error: {}", err);
            }
        }
        got_data
    }

    pub fn update_available_scans(&mut self, redis_conn: &mut redis::Connection)
    {
        let rkey = format!("{}{}", defines::KEY_BEAMLINE_AVAILABLE_SCANS, self.beamline_id);
        let mut command = command_protocols::BeamlineCommand::gen_get_avail_scans(&self.beamline_id);
        process_protocol_command(&self.client, &mut command);
        match command.reply
        {
            Some(value)=>
            {
                println!("Updating available scans for {}", self.beamline_id);
                let _: redis::RedisResult<()>= redis_conn.set(&rkey, &value);
            }
            None=> println!("Failed to get available scans for {}", self.beamline_id),
        }
    }

    /// Fetch the current scan queue from the control server and cache it in Redis.
    /// Returns the raw `queue_get` reply on success. Used on startup to seed the
    /// cache (and `last_queue_uid`) and by the live-change path.
    fn fetch_and_cache_queue(&mut self, redis_conn: &mut redis::Connection) -> Option<String>
    {
        let rkey = format!("{}{}", defines::KEY_BEAMLINE_QUEUED_SCANS, self.beamline_id);
        let mut command = command_protocols::BeamlineCommand::gen_get_queue(&self.beamline_id);
        process_protocol_command(&self.client, &mut command);
        match command.reply
        {
            Some(value)=>
            {
                let _: redis::RedisResult<()>= redis_conn.set(&rkey, &value);
                Some(value)
            }
            None=>
            {
                println!("Failed to get queued scans for {}", self.beamline_id);
                None
            }
        }
    }

    pub fn update_queued_scans(&mut self, redis_conn: &mut redis::Connection)
    {
        if let Some(reply) = self.fetch_and_cache_queue(redis_conn)
        {
            println!("Updating queued scans for {}", self.beamline_id);
            // Seed the UID so the first live change isn't reported as a redundant event.
            self.last_queue_uid = extract_queue_uid(&reply);
        }
    }

    /// Fetch the scan history (completed scans) from the control server and cache
    /// it in Redis. Returns the raw `history_get` reply on success. Used on startup
    /// to seed the cache (and `last_history_uid`) and by the live-change path.
    fn fetch_and_cache_history(&mut self, redis_conn: &mut redis::Connection) -> Option<String>
    {
        let rkey = format!("{}{}", defines::KEY_BEAMLINE_DONE_SCANS, self.beamline_id);
        let mut command = command_protocols::BeamlineCommand::gen_get_history(&self.beamline_id);
        process_protocol_command(&self.client, &mut command);
        match command.reply
        {
            Some(value)=>
            {
                let _: redis::RedisResult<()>= redis_conn.set(&rkey, &value);
                Some(value)
            }
            None=>
            {
                println!("Failed to get scan history for {}", self.beamline_id);
                None
            }
        }
    }

    pub fn update_scan_history(&mut self, redis_conn: &mut redis::Connection)
    {
        if let Some(reply) = self.fetch_and_cache_history(redis_conn)
        {
            println!("Updating scan history for {}", self.beamline_id);
            // Seed the UID so the first finished scan isn't reported as redundant.
            self.last_history_uid = extract_history_uid(&reply);
        }
    }

    /// Re-fetch the scan history and, only if its `plan_history_uid` differs from
    /// the last one we cached, refresh the cache and publish a HISTORY_EVENT so the
    /// backend forwards it over SSE. Called alongside the queue refresh when a log
    /// line indicates queue/scan activity; the UID guard means it only publishes
    /// when a scan actually finished (a new history entry was appended).
    pub fn refresh_history_if_changed(&mut self, redis_conn: &mut redis::Connection)
    {
        if let Some(reply) = self.fetch_and_cache_history(redis_conn)
        {
            let new_uid = extract_history_uid(&reply);
            // Publish when the UID changed, or when we can't read a UID (play it safe).
            if new_uid.is_none() || new_uid != self.last_history_uid
            {
                println!("Scan history changed for {}; publishing history event", self.beamline_id);
                let event_msg = format!("{}:::{}", defines::KEY_HISTORY_EVENT, reply);
                let _: redis::RedisResult<()> = redis_conn.publish(&(self.client.beamline_id_event), &event_msg);
                self.last_history_uid = new_uid;
            }
        }
    }

    /// Re-fetch the scan queue and, only if its `plan_queue_uid` differs from the
    /// last one we cached, refresh the cache and publish a QUEUE_EVENT so the
    /// backend forwards it over SSE. Called when a console log line looks like a
    /// queue change.
    pub fn refresh_queue_if_changed(&mut self, redis_conn: &mut redis::Connection)
    {
        if let Some(reply) = self.fetch_and_cache_queue(redis_conn)
        {
            let new_uid = extract_queue_uid(&reply);
            // Publish when the UID changed, or when we can't read a UID (play it safe).
            if new_uid.is_none() || new_uid != self.last_queue_uid
            {
                println!("Scan queue changed for {}; publishing queue event", self.beamline_id);
                let event_msg = format!("{}:::{}", defines::KEY_QUEUE_EVENT, reply);
                let _: redis::RedisResult<()> = redis_conn.publish(&(self.client.beamline_id_event), &event_msg);
                self.last_queue_uid = new_uid;
            }
        }
    }

    fn process_request(&mut self, cmd_str: &String) -> String
    {
        let mut beamline_cmd: command_protocols::BeamlineCommand = serde_json::from_str(&cmd_str).unwrap_or(command_protocols::BeamlineCommand::new(cmd_str));
        if beamline_cmd.is_valid()
        {
            // Only one client; handle the command if it targets this beamline.
            if *beamline_cmd.get_beamline_id() == self.beamline_id
            {
                print!("Processing: {}", cmd_str);
                process_protocol_command(&self.client, &mut beamline_cmd);
            }
            else
            {
                beamline_cmd.set_client_not_found();
            }
        }
        else 
        {
            println!("Failed to parse command: {}", cmd_str);
        }
        let done_job = serde_json::to_string(&beamline_cmd).unwrap();
        done_job
    }

    pub fn poll_cmd_queue(&mut self, redis_conn: &mut redis::Connection)
    {
        let proc_result: redis::RedisResult<Option<String>> = redis_conn.rpop(&(self.redis_key_cmd_queue_processing), None);
        match proc_result
        {
            Ok(Some(value)) => 
            {
                println!("A1 : {}", value);
                let done_value = self.process_request(&value);
                let _: redis::RedisResult<()> = redis_conn.lpush(&(self.redis_key_cmd_queue_done), &done_value);
                
            }
            Ok(None) => 
            {
                let wait_result: redis::RedisResult<Option<String>> = redis_conn.rpoplpush(&(self.redis_key_cmd_queue_waiting), &(self.redis_key_cmd_queue_processing));
                match wait_result
                {
                    Ok(Some(value)) => 
                    {
                        println!("A2: {}", value);
                        let done_value = self.process_request(&value);
                        let _: redis::RedisResult<()> = redis_conn.lpush(&(self.redis_key_cmd_queue_done), &done_value);
                        let _: redis::RedisResult<()> = redis_conn.rpop(&(self.redis_key_cmd_queue_processing), None);
                    }
                    Ok(None) => 
                    {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => 
                    {
                        eprintln!("An error occurred: {}", e);
                    }
                }
            }
            Err(e) => 
            {
                eprintln!("An error occurred: {}", e);
            }
        }
    }

    pub fn send_heartbeat(&mut self, redis_conn: &mut redis::Connection)
    {
        let now: DateTime<Utc> = Utc::now();
        let datetime_string = now.to_rfc3339();
        let _: redis::RedisResult<()> = redis_conn.set(&(self.redis_key_heartbeat), datetime_string);
    }
}
