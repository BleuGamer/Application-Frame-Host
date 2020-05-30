pub mod server 
{
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::fs::File;
    use std::io;
    use std::io::BufReader;
    use std::io::prelude::*;
    use std::borrow::Cow; // Clone on write.
    use std::process::{Command, Stdio, Child};
    use std::env;
    use serde_json;

    pub struct Server 
    {
        /*
        root_url: String,
        parent_dir: String,
        saves_dir: String,
        default_save: String,
        game_version: String,
        */
        root: PathBuf,
        parent: String,

        args: Vec<String>,
        cwd: Option<String>,
        
    }

    impl Server 
    {
        /// Creates a new 'Server' with a root directory and application subdirectories.
        /// 
        /// TODO: Full implimentation examples.
        pub fn new(root: String, parent: String) -> Server
        {
            // let raw: &str = &Self::read_file()[..];
            // let config: serde_json::Value = serde_json::from_str(raw)?;

            let server = Server
            {
                // TODO: Trait this for abstraction.
                root: PathBuf::from(root),
                parent: parent,

                args: Vec::new(),
                cwd: None,
            };

            server
        }

        pub fn arg<'a>(&'a mut self, arg: String) -> &'a mut Server
        {
            self.args.push(arg);
            self
        }

        pub fn args<'a>(&'a mut self, args: &[String]) -> &'a mut Server
        {
            self.args.extend_from_slice(args);
            self
        }

        pub fn cwd<'a>(&'a mut self, dir: String) -> &'a mut Server
        {
            self.cwd = Some(dir);
            self
        }

        pub fn show_details(self: &Self) 
        {
            // println!("Root DIR: {}", self.root_url);
            // println!("Factorio DIR: {}", self.game_dir)
        }

        fn get_main() -> io::Result<PathBuf>
        {
            let mut exe = env::current_exe()?;
            exe.set_file_name("out.txt");

            Ok(exe)
        }


        pub fn start(self: &Self)
        {

            //let exe = Self::get_main().unwrap();
            //let outputs = File::create(exe)?;
        /*
            let factorio: PathBuf = PathBuf::from(self.root_url.as_str())
                .join(self.parent_dir.as_str())
                .join(self.game_version.as_str())
                .join("bin")
                .join("x64")
                .join("factorio");

            let factorio_save: PathBuf = PathBuf::from(self.root_url.as_str())
                .join(self.parent_dir.as_str())
                .join(self.game_version.as_str())
                .join(self.saves_dir.as_str())
                .join(self.default_save.as_str());

            println!("DIR: {}", factorio.display());
            
            let fserver = Command::new(factorio)
                .arg("--start-server")
                .arg(factorio_save)
                .stdout(Stdio::from(outputs))
                .spawn()
                .unwrap();

                Ok(fserver)
        } 

        fn read_file() -> String
        {
            let mut pwd = Self::get_main().unwrap();
            pwd.set_file_name("config.json");
            let file = File::open(pwd.as_path()).expect("Could not open file.");
            let mut buffered_reader = BufReader::new(file);
            let mut contents = String::new();
            let _number_of_bytes: usize = match buffered_reader.read_to_string(&mut contents)
            {
                Ok(_number_of_bytes) => _number_of_bytes,
                Err(_err) => 0
            };
            */

            
        }
        
    }
}
