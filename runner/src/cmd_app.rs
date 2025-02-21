use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CmdArg
{
    pub name: String,
    pub desc: String,
    pub decorator: String,
    pub translator: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CmdApp
{
    pub name: String,
    pub path: String,
    pub exe: String,
    pub args: Vec<CmdArg>
}
