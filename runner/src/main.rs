use std::process::{Command};
use std::fs::{OpenOptions};
use std::io::{Result};
use chrono::{Utc};
use std::fs;
use zmq;
use std::io::{self, Write};
//use std::thread;
//use std::time::Duration;

fn atoi(s: &str) -> u64 {
    s.parse().unwrap()
}

mod cmd_app;
//mod analysis_job;

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

fn load_app_config(filename: &str) -> Result<cmd_app::CmdApp> 
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
    let app = load_app_config("../backend/runner_apps/xrf_maps.yml").unwrap();

    let context = zmq::Context::new();

    // socket to receive messages on
    let receiver = context.socket(zmq::PULL).unwrap();
    assert!(receiver.connect("tcp://localhost:5557").is_ok());

    //  Socket to send messages to
    let sender = context.socket(zmq::PUSH).unwrap();
    assert!(sender.connect("tcp://localhost:5558").is_ok());

    let controller = context.socket(zmq::SUB).unwrap();
    controller
        .connect("tcp://localhost:5559")
        .expect("failed connecting controller");
    controller.set_subscribe(b"").expect("failed subscribing");

    loop 
    {
        let mut items = [
            receiver.as_poll_item(zmq::POLLIN),
            controller.as_poll_item(zmq::POLLIN),
        ];
        zmq::poll(&mut items, -1).expect("failed polling");
        if items[0].is_readable() 
        {
            let string = receiver.recv_string(0).unwrap().unwrap();

            // Show progress
            print!(".");
            let _ = io::stdout().flush();

            // Do the work
            let res = run(&app);
            res.expect("Failed to run");
            //thread::sleep(Duration::from_millis(atoi(&string)));

            // Send results to sink
            sender.send("", 0).unwrap();
        }
        if items[1].is_readable() 
        {
            break;
        }
    }
}
