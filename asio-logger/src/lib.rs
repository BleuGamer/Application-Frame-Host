// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::fs::OpenOptions;
use slog::{Drain, o, info};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct Logger {
    output: slog::Logger,
    files: BTreeMap<&'static str, slog::Logger>,
}

impl Logger {
    pub fn new(dir: impl Into<PathBuf>) -> Logger {

        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let _out = slog::Logger::root(drain, o!());

        let mut logger = Logger {
            output: _out,
            files: BTreeMap::new(),
        };

        let log = Logger::create_file_logger("All", dir.into());

        logger.files.insert("All", log);

        logger
    }

    fn create_file_logger(name: impl Into<String>, dir: PathBuf) -> slog::Logger {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(dir.join(name.into()))
            .unwrap();
        
        let decorator = slog_term::PlainDecorator::new(file);
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, o!());

        logger
    }

    pub fn log(&self, msg: &str) -> &Self{
        info!(self.output, "{}", msg);
        info!(self.files["All"], "{}", msg);

        self
    }
}

#[macro_export]
macro_rules! log {
    ($logger:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::Logger::log($logger, &format_args!($fmt, $($arg)*).to_string())
    }
}
