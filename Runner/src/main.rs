use std::process::{Command};
use std::fs::{OpenOptions};
use std::io::{Result};
use chrono::{Utc};
use std::fs;

mod cmd_app;

fn run(app: &cmd_app::CmdApp) -> Result<()> 
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
    //println!{"{} {}", &app.path, &app.exe};
    // Define the command to execute the remote application
    let mut cmd = Command::new(&app.exe);
    cmd.current_dir(&app.path);
    cmd.stdout(stdout_file);
    cmd.stderr(stderr_file);

    // Redirect stdout and stderr to our custom files
    let mut child = cmd.spawn()?;
    
    // Wait for the process to finish
    let status = child.wait()?;
    
    // Write exit code to stderr
    println!("Process exited with code: {}", status.code().unwrap_or(-1));
    
    Ok(())
}

fn load_config(filename: &str) -> Result<cmd_app::CmdApp> 
{
    //println!{"{}",filename};
    let contents = fs::read_to_string(filename)
    .expect("Should have been able to read the file");
    //println!{"{}",contents};
    // don't unwrap like this in the real world! Errors will result in panic!
    let app_file: cmd_app::CmdApp = serde_yaml::from_str::<cmd_app::CmdApp>(&contents).unwrap();

    //println!("{:#?}", app_file);
    Ok(app_file)
}

fn main() 
{
    let app = load_config("../backend/runner_apps/xrf_maps.yml").unwrap();
    let res = run(&app);
    res.expect("Failed to run");
}
