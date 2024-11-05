use std::process::{Command};
use std::fs::{OpenOptions};
use std::io::{Result};
use chrono::{DateTime, Datelike, Timelike, Utc};

fn run() -> Result<()> 
{
    let now = Utc::now().format("%Y_%m_%d__%H_%M_%S").to_string();
    let mut stdout_name = String::from("log_");
    stdout_name.push_str(&now);
    stdout_name.push_str(".log");

    // Create files for stdout and stderr
    let stdout_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(stdout_name)?;

    let mut stderr_name = String::from("err_");
    stderr_name.push_str(&now);
    stderr_name.push_str(".log");
        
    let stderr_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(stderr_name)?;

    // Define the command to execute the remote application
    let mut cmd = Command::new("ls");
    cmd.stdout(stdout_file);
    cmd.stderr(stderr_file);

    // Redirect stdout and stderr to our custom files
    let mut child = cmd.spawn()?;
    
    // Wait for the process to finish
    let status = child.wait()?;
    
    // Write exit code to stderr
    eprintln!("Process exited with code: {}", status.code().unwrap_or(-1));
    
    Ok(())
}

fn main() 
{
    let _ = run();
}
