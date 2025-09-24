//use zmq::{self, Error, PollItem};
//use redis::{Commands, Connection};
use std::time::Duration;
use std::fs;
use clap::Parser;
//use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
//use std::thread;
use std::collections::HashMap;

mod config;
mod beamline_controls;


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
    let mut client_map = beamline_controls::ClientMap::init(&config, &context);//.expect("Error setting up zmq clients");

    let redis_client = redis::Client::open(config.redis_config.conn_str).expect("Failed to connect to Redis");
    let mut redis_conn = redis_client.get_connection().expect("Failed to get Redis connection");

    println!("Connected to Redis");
    println!("Polling for messages...");

    // Poll for messages in a loop
    loop 
    {
        // poll and publish and logs
        client_map.poll_logs(&mut redis_conn);
        // poll redis command queue
        
        // Sleep 
        std::thread::sleep(Duration::from_millis(10));
    }
}
