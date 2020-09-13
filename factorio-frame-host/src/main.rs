// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

use asio_logger;
use frame_host;

use futures::future::lazy;
use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{bounded, select, tick, Receiver};

mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    println!("LOGGER DIR: {}", parser::get_main().unwrap().display());
    let logger = asio_logger::Logger::new(parser::get_main().unwrap());
    logger.log("Test Log.");

    let server_details = &mut parser::ServerDetails::default();
    parser::read_contents(server_details).unwrap();

    let fppath = Path::new(server_details.root_url.as_ref().unwrap());
    let fpparent: PathBuf = PathBuf::from(server_details.parent_dir.as_ref().unwrap());
    let fpath: PathBuf = PathBuf::from(server_details.executable.as_ref().unwrap());
    //.join("x64").join("factorio");

    let mut fserver = frame_host::server::Server::new(fppath);
    fserver.parent(fpparent);
    fserver.child(fpath);
    fserver.output(parser::get_main().unwrap());
    fserver.show_details();
    fserver.arg("--start-server");
    //fserver.arg("/opt/factorio/1.0/saves/test.zip");
    fserver.start();

    // println!("yoooooo: {}", server_details.default_save.as_ref().unwrap());

    let tpath = std::env::current_dir()?;
    println!("PWD is {}", tpath.display());

    loop {
        select! {
            recv(ticks) -> _ =>
            {
                println!("Working!");
            }
            recv(ctrl_c_events) -> _ =>
            {
                println!("Stopping Factorio Server.");
                fserver.stop();
                break;
            }
        }
    }

    Ok(())
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}
