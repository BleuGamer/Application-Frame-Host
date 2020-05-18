
pub mod server 
{
    use std::path::{Path, PathBuf};
    use std::fs::File;
    use std::io;
    use std::process::{Command, Stdio, Child};
    use std::env;

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

        fn get_main() -> io::Result<PathBuf>
        {
            let mut exe = env::current_exe()?;
            exe.set_file_name("out.txt");

            Ok(exe)
        }

        pub fn start(self: &Self) -> Result<Child, io::Error>
        {

            let exe = Self::get_main().unwrap();

            let outputs = File::create(exe)?;

            let fserver = Command::new(self.game_dir.as_path())
                .arg("--start-server")
                .arg(self.saves_dir.join(self.save.to_string()))
                .stdout(Stdio::from(outputs))
                .spawn()
                .unwrap();

                Ok(fserver)
        }
    }
}
