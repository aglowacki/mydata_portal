use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub redis_config: Redis_Config,
    pub bluesky_clients: Vec<BS_Client>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Redis_Config {
    pub conn_str: String,
    pub username: Optional<String>,
    pub password: Optional<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BS_Client {
    pub conn_str: String,
    pub zmq_topic: String,
    pub redis_channel: String
}