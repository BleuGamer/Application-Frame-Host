use serde_json::{Result, Value};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::io::{BufRead, BufReader};
use std::io::prelude::*;
use std::io;
use std::env;
use std::fs::File;

pub fn read_contents() -> Result<String>
{
    let v: Value = serde_json::from_str(read_file().as_str())?;

    Ok(v["pig"].to_string())
}

fn read_file() -> String
{
    let mut pwd = get_main().unwrap();
    pwd.set_file_name("factorio.json");
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

pub fn get_main() -> io::Result<PathBuf>
{
    let mut exe = env::current_exe()?;
    exe.set_file_name("out.txt");
    Ok(exe)
}
