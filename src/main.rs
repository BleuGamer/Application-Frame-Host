// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate ctrlc;

use std::path::Path;
use futures::future::lazy;
use std::time::Duration;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::thread;
use std::error::Error;
use crossbeam_channel::{bounded, tick, Receiver, select};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error>
{
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ =sender.send(());
    })?;

    Ok(receiver)
}

fn factorio_server() -> Result<(), Box<dyn std::error::Error>>
{
    let fppath = Path::new("/opt/factorio");
    let fpath = fppath.join("bin").join("x64").join("factorio");
    let savepath = fppath.join("saves").join("test.zip");

    let mut fserver = Command::new(fpath).arg("--start-server").arg(savepath)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let fserver_stdout = fserver.stdout.as_mut().unwrap();
    let child_stdin = fserver.stdin.unwrap();

    

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    let mut running: bool = false;

    factorio_server();
    
    println!("Test Async.");

    loop 
    {
        select! 
        {
            recv(ticks) -> _ =>
            {
                // println!("Working!");
            }
            recv(ctrl_c_events) -> _ =>
            {
                println!("Stopping Factorio Server.");
                running = false;
                break;
            }
        }
    }

    Ok(())
}
