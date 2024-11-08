use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CmdArg
{
    name: String,
    desc: String,
    decorator: String,
    translator: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CmdApp
{
    pub name: String,
    pub path: String,
    pub exe: String,
    pub args: Vec<CmdArg>
}
