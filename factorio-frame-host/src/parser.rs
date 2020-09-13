use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

// This will eventually be agnostic.
// Currently factorio specific.
#[derive(Default)]
pub struct ServerDetails {
    pub root_url: Option<String>,
    pub parent_dir: Option<String>,
    pub executable: Option<String>,
    pub saves_dir: Option<String>,
    pub default_save: Option<String>,
}

pub fn read_contents(_sd: &mut ServerDetails) -> Result<()> {
    let v: Value = serde_json::from_str(read_file().as_str())?;

    _sd.root_url = Some(v["root_url"].as_str().unwrap().to_string());
    _sd.parent_dir = Some(v["parent_dir"].as_str().unwrap().to_string());
    _sd.executable = Some(v["executable"].as_str().unwrap().to_string());
    _sd.saves_dir = Some(v["saves_dir"].as_str().unwrap().to_string());
    _sd.default_save = Some(v["default_save"].as_str().unwrap().to_string());

    Ok(())
}

fn read_file() -> String {
    let mut pwd = get_main().unwrap();
    pwd.push("factorio.json");
    let file = File::open(pwd.as_path()).expect("Could not open file.");
    let mut buffered_reader = BufReader::new(file);
    let mut contents = String::new();
    let _number_of_bytes: usize = match buffered_reader.read_to_string(&mut contents) {
        Ok(_number_of_bytes) => _number_of_bytes,
        Err(_err) => 0,
    };
    contents
}

pub fn get_main() -> io::Result<PathBuf> {
    let mut exe = env::current_exe()?;
    exe.pop();
    Ok(exe)
}
