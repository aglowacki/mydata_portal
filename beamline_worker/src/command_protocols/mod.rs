use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tokio::task::Id;
use std::fmt;
use std::collections::{HashMap};

// base command to know which beamline to send to
#[derive(Serialize, Deserialize, Debug, )]
pub struct BeamlineCommand 
{
    // task_id: uuid
    beamline_id: String,
    pub status: String,
    cmd: String,
    args: HashMap<String,String>,
    pub username: Option<String>,
    pub reply: Option<String>,
    pub proc_start_time: Option<DateTime<Utc>>,
    pub proc_end_time: Option<DateTime<Utc>>,
    original_str: Option<String>,
}

// base command to know which beamline to send to
#[derive(Serialize, Deserialize, Debug)]
pub struct BeamlineTaskQueues 
{
    pub beamline_id: String,
    pub queued: Vec<BeamlineCommand>,
    pub processing: Vec<BeamlineCommand>,
    pub done: Vec<BeamlineCommand>,
}

impl fmt::Display for BeamlineCommand 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result 
    {
        write!(f, "beamline: {}, status: {}, username: {:?}, cmd: {}, args: {:?}", self.beamline_id, self.status, self.username, self.cmd, self.args)
    }
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
            username: None,
            reply: None,
            proc_start_time: Some(Utc::now()),
            proc_end_time: Some(Utc::now()),
            original_str: Some(cmd_str.clone()),
        }
    }
    pub fn gen_queued_from_command(beam_id: &str, user: String, command: &BeamlineCommand) -> Self
    {
        Self
        {
            beamline_id: beam_id.to_string(),
            status: defines::STR_QUEUED.to_string(),
            cmd: command.cmd.clone(),
            args: command.args.clone(),
            username: Some(user),
            reply: None,
            proc_start_time: Some(Utc::now()),
            proc_end_time: Some(Utc::now()),
            original_str: command.original_str.clone(),
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

    pub fn gen_get_avail_scans(beam_id: &String) -> Self
    {
        let mut arg_hash = HashMap::new();
        arg_hash.insert("user_group".to_string(), "primary".to_string());
        Self
        {
            beamline_id: beam_id.clone(),
            status: "QUEUED".to_string(),
            cmd: "plans_allowed".to_string(),
            args: arg_hash,
            username: None,
            reply: None,
            proc_start_time: None,
            proc_end_time: None,
            original_str: None,
        }
    }

}

impl BeamlineTaskQueues
{
    pub fn new(Id: &String) -> Self
    {
        Self
        {
            beamline_id: Id.clone(),
            queued: Vec::new(),
            processing: Vec::new(),
            done: Vec::new(),
        }
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
        //println!("Client sent: {}", message);

        // Set a receive timeout of 3 seconds (3000 milliseconds)
        channel.set_rcvtimeo(30000)?; 
        match channel.recv_bytes(0) 
        {
            Ok(reply) => 
            {
                let reply_str = String::from_utf8_lossy(&reply).to_string();
                //println!("Client received: {}", reply_str);
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