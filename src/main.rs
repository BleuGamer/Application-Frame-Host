// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::path::Path;
use futures::future::lazy;
use std::time::Duration;
use std::process::Child;
use std::io::{BufRead, BufReader};
use std::thread;
use std::error::Error;
use crossbeam_channel::{bounded, tick, Receiver, select};

mod ffh;
use ffh::framehost::server;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error>
{
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ =sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    let fppath = Path::new("/opt/factorio");
    let fpath = fppath.join("bin").join("x64").join("factorio");
    let savepath = fppath.join("saves");
    
    /*
    let fserver = ffh::framehost::server::FactorioServer
    {
        parent_dir: fppath.to_path_buf(),
        game_dir: fpath.to_path_buf(),
        saves_dir: savepath.to_path_buf(),

        default_save: "test.zip".to_string(),
        
        game_version: "0.18.24".to_string()
    };
    */
    let fserver = server::FactorioServer::new();
    fserver.unwrap().show_details();
    // let handle = fserver.start();

    let tpath = std::env::current_dir()?;
    println!("PWD is {}", tpath.display());

    loop 
    {
        select! 
        {
            recv(ticks) -> _ =>
            {
                println!("Working!");
            }
            recv(ctrl_c_events) -> _ =>
            {
                println!("Stopping Factorio Server.");
                //handle.unwrap().kill().expect("Factorio isn't running.");
                break;
            }
        }
    }

    Ok(())
}
