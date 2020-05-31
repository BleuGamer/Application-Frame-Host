pub mod server 
{
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio, Child};
    use std::io::Write;

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

            assert!(server.root.is_absolute());
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

        pub fn child<'a>(&'a mut self, child: impl Into<PathBuf>) -> &'a mut Server
        {
            self.child = Some(child.into());
            self
        }

        pub fn cwd<'a>(&'a mut self, dir: String) -> &'a mut Server
        {
            self.cwd = Some(dir);
            self
        }

        pub fn start<'a>(&'a mut self) -> &'a mut Server
        {
            let child: PathBuf = self.root
                .join(self.parent.as_ref().unwrap())
                .join(self.child.as_ref().unwrap());

            self.handle = Some(Command::new(child)
                .args(&self.args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap());

            self
        }

        pub fn stop<'a>(&'a mut self) -> &'a mut Server
        {
            // TODO: Access handle.

            self
        }
    }
}

