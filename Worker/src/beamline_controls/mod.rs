use zmq::PollItem;
use redis::{Commands};
use std::{collections::HashMap};
use std::sync::Arc;
use crate::config;

pub struct BsClient
{
    pub cmd_address: String,
    pub log_address: String,
    pub topic: String,
    pub redis_key: String,
    pub subscriber: zmq::Socket,
}

impl BsClient 
{
    pub fn new(config: &config::BsClient, context: &zmq::Context) -> Self
    {
        Self 
        {
            cmd_address: format!("tcp://{}:60615", config.host.to_string()),
            log_address: format!("tcp://{}:60625", config.host.to_string()),
            topic: String::from(config.zmq_topic.clone()),
            redis_key: String::from(config.redis_channel.clone()),
            subscriber: context.socket(zmq::SUB).expect("Failed to create SUB socket"),
        }
    }

    pub fn connect(&self)
    {
        self.subscriber.connect(&self.log_address).expect("Failed to connect to PUB socket");
        println!("Connecting to ZeroMQ PUB at {}", self.log_address);
        self.subscriber.set_subscribe(self.topic.as_bytes()).expect("Failed to subscribe");
    }

    pub fn gen_poll_item(&self) -> PollItem<'_>
    {
        return self.subscriber.as_poll_item(zmq::POLLIN);
    }


}

pub struct ClientMap
{
    client_map: HashMap<String, Arc<BsClient>>,
    poll_map: HashMap<usize, Arc<BsClient>>,
    poll_list: Vec<PollItem<'static>>,
}

impl ClientMap
{
    pub fn init(config: &config::Config, context: &zmq::Context ) -> Self
    {
        let mut c_map:HashMap<String, Arc<BsClient>> = HashMap::new();
        let mut p_map:HashMap<usize, Arc<BsClient>> = HashMap::new();
        let mut p_list:Vec<PollItem<'static>> = Vec::new();
        let mut idx: usize = 0;
        for bs_client_config in config.bluesky_clients.iter()
        {
            let bs_client = Arc::new(BsClient::new(bs_client_config, context));
            bs_client.connect();
            // SAFETY: We extend the lifetime to 'static because Arc ensures the data lives long enough.
            let poll_item: PollItem<'static> = unsafe { std::mem::transmute(bs_client.gen_poll_item()) };
            p_list.push(poll_item);
            c_map.insert(bs_client_config.redis_channel.to_string(), Arc::clone(&bs_client));
            p_map.insert(idx, Arc::clone(&bs_client));
            idx = idx + 1;
        }

        Self
        {
            client_map: c_map,
            poll_map: p_map,
            poll_list: p_list,
        }
    }

    pub fn get_client_by_name(&self, name: &str) -> Option<Arc<BsClient>>
    {
        self.client_map.get(name).cloned()
    }
    /*
    pub fn get_client_by_pollitem(&self, id: PollItem) -> Option<&BsClient>
    {
        self.poll_map.get(id)
    }
    */
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
                                    if message != client.topic
                                    {
                                        // Push the message to the Redis list
                                        let result: redis::RedisResult<()> = redis_conn.rpush(&(client.redis_key), &message);
                                        match result 
                                        {
                                            Ok(_) => println!("Message pushed to Redis list: {}", client.redis_key),
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
}
