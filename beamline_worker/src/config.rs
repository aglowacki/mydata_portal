use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config
{
    pub redis_config: RedisConfig,
    pub control_clients: Vec<ControlClient>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisConfig 
{
    pub conn_str: String,
    pub redis_cmd_queue: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ControlClient
{
    pub host: String,
    pub zmq_log_topic: String,
    pub beamline_id: String,
    pub protocol: String,
}