use std::process::{Command};
use std::fs::{OpenOptions};
use std::io::{Result};
use chrono::{Utc};

fn run() -> Result<()> 
{
    let now_str = Utc::now().format("%Y_%m_%d__%H_%M_%S").to_string();
    let stdout_name = format!("prog_{}.log",now_str);
    let stderr_name = format!("prog_{}.err",now_str);

    // Create files for stdout and stderr
    let stdout_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(stdout_name)?;
        
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
