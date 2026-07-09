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
                    let client = &self.client;
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
                                let _: redis::RedisResult<()> = redis_conn.publish(&(client.beamline_id_log), &message);
                                match result
                                {
                                    Ok(_) => println!("Message pushed to Redis list: {}", client.beamline_id_log),
                                    Err(err) => eprintln!("Failed to push message to Redis: {}", err),
                                }
                            }
                        }
                        Ok(Err(err)) => eprintln!("Failed to decode message: {}", String::from_utf8(err).expect("Failed to decode!")),
                        Err(err) => eprintln!("Failed to receive message: {}", err),
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
