// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate slog;
use slog::Drain;
use slog_term;
use asio_logger;
use frame_host;
use util;
use web_api;

use futures::future::lazy;
use std::borrow::Cow;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{bounded, select, tick, Receiver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = asio_logger::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());

    //logger.log_msg("STARTING SERVER");

    let server_details = &mut util::parser::ServerDetails::default();
    util::parser::read_contents("factorio.json", server_details).unwrap();

    let fppath = Path::new(server_details.root_url.as_ref().unwrap());
    let fpparent: PathBuf = PathBuf::from(server_details.parent_dir.as_ref().unwrap());
    let fpath: PathBuf = PathBuf::from(server_details.executable.as_ref().unwrap());
    //.join("x64").join("factorio");

    let mut fserver = frame_host::server::Server::new( fppath);
    fserver.parent(fpparent);
    fserver.child(fpath);
    let mut output = util::env::get_cwd().unwrap();
    output.push("out.txt");
    fserver.output(output);

    info!(&logger, "Root: {}", fserver.root.display());
    info!(
        &logger,
        "Parent: {}",
        fserver.parent.as_mut().unwrap().display()
    );
    info!(
        &logger,
        "child: {}",
        fserver.child.as_mut().unwrap().display()
    );
    info!(
        &logger,
        "Output File: {}",
        fserver.output.as_mut().unwrap().display()
    );

    fserver.arg("--start-server");
    //fserver.arg("/opt/factorio/1.0/saves/test.zip");
    //fserver.start();

    let tpath = std::env::current_dir()?;
    info!(&logger, "PWD: {}", tpath.display());

    std::thread::spawn(move || {
        // This is unsafe.
        // Temporary testing.
        // TODO: Proper Actix Async handling.
        let waw = web_api::start();
        let was = web_api::start_web_socket();
    });

    loop {
        select! {
            recv(ticks) -> _ =>
            {
                info!(&logger, "Working!");
            }
            recv(ctrl_c_events) -> _ =>
            {
                info!(&logger, "Stopping Factorio Server.");
                // fserver.stop();
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
