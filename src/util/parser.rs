use serde_json::{Result, Value};
use std::fs::{write, File};
use std::io::prelude::*;
use std::io::BufReader;

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

pub fn read_contents(file: &str, _sd: &mut ServerDetails) -> Result<()> {
    let v: Value = serde_json::from_str(read_cwd_file(file).as_str())?;

    _sd.root_url = Some(v["root_url"].as_str().unwrap().to_string());
    _sd.parent_dir = Some(v["parent_dir"].as_str().unwrap().to_string());
    _sd.executable = Some(v["executable"].as_str().unwrap().to_string());
    _sd.saves_dir = Some(v["saves_dir"].as_str().unwrap().to_string());
    _sd.default_save = Some(v["default_save"].as_str().unwrap().to_string());

    Ok(())
}

pub fn read_json_member(file: &str, member: &str) -> Result<String> {
    let v: Value = serde_json::from_str(read_cwd_file(file).as_str())?;

    Ok(v[member].as_str().unwrap().to_string())
}

// TODO: Expand upon for verification.
pub fn write_json_file(file: &str, json: String) {
    write(file, &json).expect("Unable to write file.");
}

fn read_cwd_file(file: &str) -> String {
    let mut pwd = crate::env::get_cwd().unwrap();
    println!("{}", pwd.display());
    pwd.push(file);
    println!("{}", pwd.display());
    let file = File::open(pwd.as_path()).expect("Could not open file.");
    let mut buffered_reader = BufReader::new(file);
    let mut contents = String::new();
    let _number_of_bytes: usize = match buffered_reader.read_to_string(&mut contents) {
        Ok(_number_of_bytes) => _number_of_bytes,
        Err(_err) => 0,
    };
    contents
}
