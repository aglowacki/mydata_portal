use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::{HashMap};

// base command to know which beamline to send to
#[derive(Serialize, Deserialize, Debug)]
pub struct BeamlineCommand 
{
    beamline_id: String,
    //queue_time: String,
    pub status: String,
    cmd: String,
    args: HashMap<String,String>,
    pub reply: Option<String>,
    pub proc_start_time: Option<DateTime<Utc>>,
    pub proc_end_time: Option<DateTime<Utc>>,
    original_str: Option<String>,
}

impl BeamlineCommand
{
    pub fn new(cmd_str: &String) -> Self
    {
        Self
        {
            beamline_id: "N/A".to_string(),
            status: "BAD_CMD".to_string(),
            cmd: "".to_string(),
            args: HashMap::new(),
            reply: None,
            proc_start_time: Some(Utc::now()),
            proc_end_time: Some(Utc::now()),
            original_str: Some(cmd_str.clone()),
        }
    }

    pub fn is_valid(&self) -> bool
    {
        if self.status == "BAD_CMD"
        {
            return false;
        }
        true
    }

    pub fn get_beamline_id(&self) -> &String
    {
        &(self.beamline_id)
    }

    pub fn set_client_not_found(&mut self)
    {
        self.status = "CONTROL_CLIENT_NOT_FOUND".to_string();
    }

    pub fn set_unable_to_translate_protocol(&mut self)
    {
        self.status = "UNABLE_TO_TRANS_PROTOCOL".to_string();
    }
    

}

pub trait Command
{
    fn execute(&mut self, channel: &zmq::Socket) -> Result<String, zmq::Error>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlueSkyCommand
{
    method: String,
    params: HashMap<String,String>, 
}

impl BlueSkyCommand
{
    pub fn new(cmd: &String, args: &HashMap<String,String>) -> Self
    {
        Self
        {
            method: cmd.clone(),
            params: args.clone(),
        }
    }
}

impl Command for BlueSkyCommand
{
    fn execute(&mut self, channel: &zmq::Socket) -> Result<String, zmq::Error>
    {
        let message = serde_json::to_string(&self).unwrap();
        channel.send(message.as_bytes(), 0)?;
        println!("Client sent: {}", message);

        // Set a receive timeout of 3 seconds (3000 milliseconds)
        channel.set_rcvtimeo(30000)?; 
        match channel.recv_bytes(0) 
        {
            Ok(reply) => 
            {
                let reply_str = String::from_utf8_lossy(&reply).to_string();
                println!("Client received: {}", reply_str);
                return Ok(reply_str);
            }
            Err(zmq::Error::EAGAIN) => 
            {
                return Err(zmq::Error::EAGAIN);
            }
            Err(e) => 
            {
                return Err(e);
            }
        }
    }
}

pub fn generate_cmd(protocol: &String, beamline_cmd: &BeamlineCommand) -> Option<Box<dyn Command>>
{
    if protocol == "BlueSky"
    {
        return Some(Box::new(BlueSkyCommand::new(&beamline_cmd.cmd, &beamline_cmd.args)));
    }
    None
}