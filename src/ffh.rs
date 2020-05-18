
pub mod server 
{
    use std::path::{Path, PathBuf};
    use std::fs::File;
    use std::io::Error;
    use std::process::{Command, Stdio, Child};

    // These members should probably be encapsulated eventually.
    pub struct FactorioServer 
    {
        pub parent_dir: PathBuf,
        pub game_dir: PathBuf,
        pub saves_dir: PathBuf,

        pub save: String,

        pub game_version: String,
    }

    impl FactorioServer 
    {
        pub fn show_details(self: &Self) 
        {
            println!("Factorio version: {}", self.game_version);
            // println!("Factorio DIR: {}", self.game_dir)
        }

        pub fn start(self: &Self) -> Result<Child, Error>
        {
            let outputs = File::create("out.txt")?;

            let mut fserver = Command::new(self.game_dir.as_path())
                .arg("--start-server")
                .arg(self.saves_dir.join(self.save.to_string()))
                .stdout(Stdio::from(outputs))
                .spawn()
                .unwrap();

                // let fserver_stdout = fserver.stdout.as_mut().unwrap();

                

                Ok(fserver)
        }
    }
}
