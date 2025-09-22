use zmq::{self, Error, PollItem};
use redis::{Commands, Connection};
use std::time::Duration;
use std::fs;
use clap::Parser;

mod config;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args 
{
    /// How deep to search for datasets
    #[arg(short, long, default_value="config.json")]
    config_filename: String,
}

struct BsClient
{
    address: String,
    topic: String,
    redis_key: String,
    subscriber: zmq::Socket,
}

impl BsClient 
{
    fn new(config: &config::BsClient, context: &zmq::Context) -> Self
    {
        Self 
        {
            address: String::from(config.conn_str.clone()),
            topic: String::from(config.zmq_topic.clone()),
            redis_key: String::from(config.redis_channel.clone()),
            subscriber: context.socket(zmq::SUB).expect("Failed to create SUB socket"),
        }
    }

    fn connect(&self)
    {
        self.subscriber.connect(&self.address).expect("Failed to connect to PUB socket");
        println!("Connecting to ZeroMQ PUB at {}", self.address);
        self.subscriber.set_subscribe(self.topic.as_bytes()).expect("Failed to subscribe");
    }

    fn gen_poll_item(&self) -> PollItem<'_>
    {
        return self.subscriber.as_poll_item(zmq::POLLIN);
    }


}

fn setup_zmq_connections(config: &config::Config, context: &zmq::Context ) -> Vec<BsClient>
{
    let mut client_list = Vec::new();
    for bs_client_config in config.bluesky_clients.iter()
    {
        let bs_client = BsClient::new(bs_client_config, context);
        bs_client.connect();
        client_list.push(bs_client);
    }
    client_list
}

fn main() 
{
    let args = Args::parse();
    // Read the file content as a string
    let config_content = fs::read_to_string(args.config_filename).expect("Error loading config file.");
    // Deserialize the YAML string into the Config struct
    let config: config::Config = serde_json::from_str(&config_content).expect("Error parsing config file.");
    // ZeroMQ context and subscriber socket
    let context = zmq::Context::new();
    let clients = setup_zmq_connections(&config, &context);//.expect("Error setting up zmq clients");

    let redis_client = redis::Client::open(config.redis_config.conn_str).expect("Failed to connect to Redis");
    let mut redis_conn = redis_client.get_connection().expect("Failed to get Redis connection");

    println!("Connected to Redis");

    // Redis list key
//    let redis_list_key = "zmq_messages";
    
    // Polling setup
    let mut poll_items = Vec::new();
    for client in clients.iter()
    {
        poll_items.push(client.gen_poll_item());
    }
    let poll_timeout = 100; // Poll timeout in milliseconds

    println!("Polling for messages...");

    // Poll for messages in a loop
    loop 
    {
        // Poll the socket
        let poll_result = zmq::poll(&mut poll_items, poll_timeout);

        match poll_result 
        {
            Ok(_) => 
            {
                for (index, poll_item) in poll_items.iter_mut().enumerate() 
                {
                    if poll_item.is_readable()
                    {
                        match clients[index].subscriber.recv_string(0)
                        {
                            Ok(Ok(message)) => 
                            {
                                println!("Received message: {}", message);
                                if message != clients[index].topic
                                {
                                    // Push the message to the Redis list
                                    let result: redis::RedisResult<()> = redis_conn.rpush(&(clients[index].redis_key), &message);
                                    match result 
                                    {
                                        Ok(_) => println!("Message pushed to Redis list: {}", clients[index].redis_key),
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
            Err(err) => {
                eprintln!("Polling error: {}", err);
            }
        }

        // Optional: Add a small sleep to avoid busy-waiting
        std::thread::sleep(Duration::from_millis(10));
    }
}
