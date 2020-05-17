// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate ctrlc;

use std::path::Path;
use tokio::prelude::*;
use tokio::process::Command;
use tokio::task;
use tokio::sync::mpsc;
use tokio::prelude::*;
use futures::future::lazy;
use std::time::Duration;
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

#[tokio::main]
async fn factorio_server(mut receiver: mpsc::Receiver<u32>) -> Result<(), Box<dyn std::error::Error>>
{
    let mut should_run: bool = true;

    let fppath = Path::new("/opt/factorio");
    let fpath = fppath.join("bin").join("x64").join("factorio");
    let savepath = fppath.join("saves").join("test.zip");

    let fserver = Command::new(fpath).arg("--start-server").arg(savepath).output();
    // let output = fserver.await;

    receiver.recv().await;

    Ok(())
}

//#[tokio::main]
//async fn main() -> Result<(), Box<dyn std::error::Error>>
fn main() -> Result<(), Box<dyn std::error::Error>>
{
    

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    let mut running: bool = false;

    let (mut tx, mut rx) = mpsc::channel(100);

    //tokio::spawn(async move {
        factorio_server(rx);
    //});
    

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

                tokio::spawn(async move {
                    tx.send(1).await.unwrap();
                });

                running = false;
                break;
            }
        }
    }

    Ok(())
}
