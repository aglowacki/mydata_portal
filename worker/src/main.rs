//use zmq::{self, Error, PollItem};
//use redis::{Commands, Connection};
use std::time::Duration;
use std::fs;
use clap::Parser;
//use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
//use std::thread;
use std::collections::HashMap;
use std::time::Instant;

mod config;
mod beamline_controls;
mod command_protocols;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args 
{
    /// How deep to search for datasets
    #[arg(short, long, default_value="config.json")]
    config_filename: String,
}

fn main() 
{
    let args = Args::parse();
    // Read the file content as a string
    let config_content = fs::read_to_string(args.config_filename).expect("Error loading config file.");
    // Deserialize the YAML string into the Config struct
    let config: config::Config = serde_json::from_str(&config_content).expect("Error parsing config file.");
    // ZeroMQ context and subscriber socket
    //let running = Arc::new(AtomicBool::new(true));
    //let r = running.clone();
    let context = zmq::Context::new();
    println!("Connected to Redis: {}", config.redis_config.conn_str);
    let mut client_map = beamline_controls::ClientMap::init(&config, &context);//.expect("Error setting up zmq clients");

    let redis_client = redis::Client::open(config.redis_config.conn_str).expect("Failed to connect to Redis");
    let mut redis_conn = redis_client.get_connection().expect("Failed to get Redis connection");
    
    println!("Setting up redis command queue waiting {}", client_map.redis_key_cmd_queue_waiting);
    println!("Setting up redis command queue processing {}", client_map.redis_key_cmd_queue_processing);
    println!("Setting up redis command queue done {}", client_map.redis_key_cmd_queue_done);

    
    println!("Updating Redis with available scans");
    client_map.update_available_scans(&mut redis_conn);

    let mut heartbeat_start = Instant::now();
    let ten_seconds_dur = Duration::new(10, 0);
    
    println!("Polling for messages...");
    // Poll for messages in a loop
    loop 
    {
        // poll and publish and logs
        client_map.poll_logs(&mut redis_conn);
        // poll redis command queue
        client_map.poll_cmd_queue(&mut redis_conn);
        // check if we should set heartbeat
       if heartbeat_start.elapsed() > ten_seconds_dur
       {
            client_map.send_heartbeat(&mut redis_conn);
            heartbeat_start = Instant::now();
       }
    }
}
