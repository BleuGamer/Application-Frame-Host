use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn get_pwd() -> io::Result<PathBuf> {
    let mut pwd = env::current_exe()?;
    pwd.pop();
    Ok(pwd)
}

pub fn get_main() -> io::Result<PathBuf> {
    let exe = env::current_exe()?;
    Ok(exe)
}