use std::env;
use std::io;
use std::path::PathBuf;

pub fn get_cwd() -> io::Result<PathBuf> {
    let mut pwd = env::current_exe()?;
    pwd.pop();
    Ok(pwd)
}

pub fn get_main() -> io::Result<PathBuf> {
    let exe = env::current_exe()?;
    Ok(exe)
}
