// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

use frame_host;

use std::path::{Path, PathBuf};
use futures::future::lazy;
use std::time::Duration;
use std::process::Child;
use std::io::{BufRead, BufReader};
use std::thread;
use std::error::Error;
use std::fs::File;
use std::borrow::Cow;
use std::io;
use std::io::prelude::*;
use std::env;

use crossbeam_channel::{bounded, tick, Receiver, select};

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));

    let fppath = Path::new("/opt/factorio");
    let fpath = fppath.join("bin").join("x64").join("factorio");
    let savepath = fppath.join("saves");
    

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

fn get_main() -> io::Result<PathBuf>
{
    let mut exe = env::current_exe()?;
    exe.set_file_name("out.txt");
    Ok(exe)
}


fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error>
{
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ =sender.send(());
    })?;

    Ok(receiver)
}

fn read_file() -> String
{
    let mut pwd = get_main().unwrap();
    pwd.set_file_name("config.json");
    let file = File::open(pwd.as_path()).expect("Could not open file.");
    let mut buffered_reader = BufReader::new(file);
    let mut contents = String::new();
    let _number_of_bytes: usize = match buffered_reader.read_to_string(&mut contents)
    {
        Ok(_number_of_bytes) => _number_of_bytes,
        Err(_err) => 0
    };
    contents
}
