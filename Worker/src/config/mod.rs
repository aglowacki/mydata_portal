use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config
{
    pub redis_config: RedisConfig,
    pub bluesky_clients: Vec<BsClient>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisConfig 
{
    pub conn_str: String,
    pub redis_cmd_queue: String,
    pub username: Option<String>,
    pub password: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BsClient
{
    pub host: String,
    pub zmq_topic: String,
    pub redis_channel: String
}