pub mod server 
{
    use std::path::PathBuf;
    use std::process::{Command, Stdio, Child};
    use std::io::Write;
    use std::fs::File;

    pub struct Server 
    {
        root: PathBuf,
        parent: Option<PathBuf>,
        child: Option<PathBuf>,

        output: Option<PathBuf>,

        args: Vec<String>,
        cwd: Option<String>,

        handle: Option<Child>,
        
    }

    impl Server 
    {
        /// Creates a new 'Server' with a root directory and application subdirectories.
        /// 
        /// TODO: Full implimentation examples.
        pub fn new(root: impl Into<PathBuf>) -> Server
        {
            // let raw: &str = &Self::read_file()[..];
            // let config: serde_json::Value = serde_json::from_str(raw)?;

            let server = Server
            {
                // TODO: Trait this for abstraction.
                root: root.into(),
                parent: None,
                child: None,

                output: None,

                args: Vec::new(),
                cwd: None,

                handle: None,
            };

            //assert!(server.root.is_absolute());
            server
        }

        pub fn show_details(&mut self) -> &mut Self
        {
            println!("Root: {}", self.root.display());
            println!("Parent: {}", self.parent.as_mut().unwrap().display());
            println!("child: {}", self.child.as_mut().unwrap().display());

            self
        }

        pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self
        {
            self.args.push(arg.into());
            self
        }

        pub fn args(&mut self, args: &[String]) -> &mut Self
        {
            self.args.extend_from_slice(args);
            self
        }

        pub fn parent(&mut self, parent: impl Into<PathBuf>) -> &mut Self
        {
            self.parent = Some(parent.into());
            self
        }

        pub fn child(&mut self, child: impl Into<PathBuf>) -> &mut Self
        {
            self.child = Some(child.into());
            self
        }

        pub fn output(&mut self, output: impl Into<PathBuf>) -> &mut Self
        {
            self.output = Some(output.into());
            self
        }

        pub fn cwd(&mut self, dir: String) -> &mut Self
        {
            self.cwd = Some(dir);
            self
        }

        pub fn start(&mut self) -> &mut Self
        {
            let outputs = File::create(self.output.as_mut().unwrap());

            let child: PathBuf = self.root
                .join(self.parent.as_ref().unwrap())
                .join(self.child.as_ref().unwrap());

            self.handle = Some(Command::new(child)
                .args(&self.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::from(outputs.unwrap()))
                .spawn()
                .unwrap());

            self
        }

        pub fn stop(&mut self) -> &mut Self
        {
            // TODO: Access handle.
            match self.handle.as_mut().unwrap().stdin.as_mut().unwrap().write_all("/quit".as_bytes())
            {
                Ok(_n) => {}
                Err(_error) => println!("Server is not running.")
            }

            self
        }
    }
}

