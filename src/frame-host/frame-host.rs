pub mod server {
    use asio_logger;
    use asio_logger::{info, error};
    use util;

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::sync::Arc;

    pub struct Server {
        pub root: PathBuf,
        pub parent: Option<PathBuf>,
        pub child: Option<PathBuf>,

        pub output: Option<PathBuf>,

        pub args: Vec<String>,
        pub cwd: Option<String>,

        pub handle: Option<Child>,

        logger: asio_logger::Context,
    }

    impl Server {
        /// Creates a new 'Server' with a root directory and application subdirectories.
        ///
        /// TODO: Full implimentation examples.
        pub fn new(logging: Arc<asio_logger::Logger>, root: impl Into<PathBuf>) -> Server {
            // let raw: &str = &Self::read_cwd_file()[..];
            // let config: serde_json::Value = serde_json::from_str(raw)?;

            let mut logger = asio_logger::Context::new(logging, util::env::get_cwd().unwrap());
            logger.file(util::env::get_cwd().unwrap(), "log-frame-host.txt");
            info!(&logger, "Initializing frame-host!");

            let server = Server {
                // TODO: Trait this for abstraction.
                root: root.into(),
                parent: None,
                child: None,

                output: None,

                args: Vec::new(),
                cwd: None,

                handle: None,

                logger: logger,
            };

            assert!(server.root.is_absolute());
            server
        }

        pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
            self.args.push(arg.into());
            self
        }

        pub fn args(&mut self, args: &[String]) -> &mut Self {
            self.args.extend_from_slice(args);
            self
        }

        pub fn parent(&mut self, parent: impl Into<PathBuf>) -> &mut Self {
            self.parent = Some(parent.into());
            self
        }

        pub fn child(&mut self, child: impl Into<PathBuf>) -> &mut Self {
            self.child = Some(child.into());
            self
        }

        pub fn output(&mut self, output: impl Into<PathBuf>) -> &mut Self {
            self.output = Some(output.into());
            self
        }

        pub fn cwd(&mut self, dir: String) -> &mut Self {
            self.cwd = Some(dir);
            self
        }

        pub fn start(&mut self) -> &mut Self {
            let outputs = File::create(self.output.as_mut().unwrap());

            let child: PathBuf = self
                .root
                .join(self.parent.as_ref().unwrap())
                .join(self.child.as_ref().unwrap());

            self.handle = Some(
                Command::new(child)
                    .args(&self.args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::from(outputs.unwrap()))
                    .spawn()
                    .unwrap(),
            );

            self
        }

        pub fn stop(&mut self) -> &mut Self {
            // TODO: Access handle.
            match self
                .handle
                .as_mut()
                .unwrap()
                .stdin
                .as_mut()
                .unwrap()
                .write_all("/quit".as_bytes())
            {
                Ok(_n) => {}
                Err(_error) => error!(&self.logger, "Server is not running."),
            }

            self
        }
    }
}
