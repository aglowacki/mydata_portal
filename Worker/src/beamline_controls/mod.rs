use zmq::PollItem;
use redis::{Commands};
use std::{collections::HashMap};
use std::sync::Arc;
use crate::{beamline_controls, config};
use std::time::Duration;
use chrono::{DateTime, Utc};

use crate::{command_protocols};

pub struct ControlClient
{
    cmd_address: String,
    log_address: String,
    log_topic: String,
    bealine_id: String,
    protocol: String,
    subscriber: zmq::Socket,
    cmd_channel: zmq::Socket,
}

impl ControlClient 
{
    pub fn new(config: &config::ControlClient, context: &zmq::Context) -> Self
    {
        Self 
        {
            cmd_address: format!("tcp://{}:60615", config.host.to_string()),
            log_address: format!("tcp://{}:60625", config.host.to_string()),
            log_topic: String::from(config.zmq_log_topic.clone()),
            protocol: String::from(config.protocol.clone()),
            bealine_id: String::from(config.beamline_id.clone()),
            subscriber: context.socket(zmq::SUB).expect("Failed to create SUB socket"),
            cmd_channel: context.socket(zmq::REQ).expect("Failed to create CMD socket"),
        }
    }

    pub fn connect(&self)
    {
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
    client_map: HashMap<String, Arc<ControlClient>>,
    poll_map: HashMap<usize, Arc<ControlClient>>,
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
        let mut c_map:HashMap<String, Arc<ControlClient>> = HashMap::new();
        let mut p_map:HashMap<usize, Arc<ControlClient>> = HashMap::new();
        let mut p_list:Vec<PollItem<'static>> = Vec::new();
        let mut idx: usize = 0;
        for bs_client_config in config.control_clients.iter()
        {
            let bs_client = Arc::new(ControlClient::new(bs_client_config, context));
            println!("New BlueSky Client: Cmd: {} , Log: {}", bs_client.cmd_address, bs_client.log_address);
            bs_client.connect();
            // SAFETY: We extend the lifetime to 'static because Arc ensures the data lives long enough.
            let poll_item: PollItem<'static> = unsafe { std::mem::transmute(bs_client.gen_poll_item()) };
            p_list.push(poll_item);
            c_map.insert(bs_client_config.beamline_id.clone(), Arc::clone(&bs_client));
            p_map.insert(idx, Arc::clone(&bs_client));
            idx = idx + 1;
        }

        Self
        {
            redis_key_cmd_queue_waiting: format!("{}_waiting", config.redis_config.redis_cmd_queue.to_string()),
            redis_key_cmd_queue_processing: format!("{}_processing", config.redis_config.redis_cmd_queue.to_string()),
            redis_key_cmd_queue_done: format!("{}_done", config.redis_config.redis_cmd_queue.to_string()),
            client_map: c_map,
            poll_map: p_map,
            poll_list: p_list,
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
                for (index, poll_item) in self.poll_list.iter().enumerate() 
                {
                    if poll_item.is_readable()
                    {
                        if let Some(client) = self.poll_map.get(&index)
                        {
                            match client.subscriber.recv_string(0)
                            {
                                Ok(Ok(message)) => 
                                {
                                    println!("Received message: {}", message);
                                    got_data = true;
                                    if message != client.log_topic
                                    {
                                        // Push the message to the Redis list
                                        let result: redis::RedisResult<()> = redis_conn.rpush(&(client.bealine_id), &message);
                                        let _: redis::RedisResult<()> = redis_conn.publish(&(client.bealine_id), &message);
                                        match result 
                                        {
                                            Ok(_) => println!("Message pushed to Redis list: {}", client.bealine_id),
                                            Err(err) => eprintln!("Failed to push message to Redis: {}", err),
                                        }
                                    }
                                }
                                Ok(Err(err)) => eprintln!("Failed to decode message: {}", String::from_utf8(err).expect("Failed to decode!")),
                                Err(err) => eprintln!("Failed to receive message: {}", err),
                            }
                        }
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
        for (name, client) in self.client_map.iter_mut()
        {
            let rkey = format!("{}_AVAILABLE_SCANS",name);
            let mut command = command_protocols::BeamlineCommand::gen_get_avail_scans(name);
            process_protocol_command(&client, &mut command);
            match command.reply
            {
                Some(value)=>
                {
                    println!("Updating available scans for {}", name);
                    let _: redis::RedisResult<()>= redis_conn.set(&rkey, &value);
                }
                None=> println!("Failed to get available scans for {}", name),
            }
        }
    }

    fn process_request(&mut self, cmd_str: &String) -> String
    {
        let mut beamline_cmd: command_protocols::BeamlineCommand = serde_json::from_str(&cmd_str).unwrap_or(command_protocols::BeamlineCommand::new(cmd_str));
        if beamline_cmd.is_valid()
        {
            let result = self.client_map.get(beamline_cmd.get_beamline_id());
            match result
            {
                Some(cmd_client) => 
                {
                    print!("Processing: {}", cmd_str);
                    process_protocol_command(cmd_client, &mut beamline_cmd);
                }
                None => beamline_cmd.set_client_not_found()
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
}
