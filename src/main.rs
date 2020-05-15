// #![allow(stable_features)]


use std::path::Path;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{

    let fppath = Path::new("/opt/factorio");
    let fpath = fppath.join("bin").join("x64").join("factorio");
    let savepath = fppath.join("saves").join("test.zip");

    println!("{}", fpath.display());

    let child = Command::new(fpath).arg("--start-server").arg(savepath).spawn();
    let future = child.expect("failed to spawn");
    let status = future.await?;
    println!("The command exited with: {}", status);
    Ok(())
}
