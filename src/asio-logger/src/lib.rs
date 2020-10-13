// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

use slog::debug as _debug;
use slog::error as _error;
use slog::info as _info;
use slog::trace as _trace;
use slog::warn as _warn;

use slog::{o, Drain};
use std::collections::BTreeMap;
use std::fmt::write;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::RwLock;
use std::ops::DerefMut;
use std::sync::Arc;

pub struct Context {
    logger: Arc<Logging>,
    files: Vec<String>,
}

impl Context {
    pub fn new(logger: Arc<Logging>, dir: impl Into<PathBuf>) -> Self {
        let context = Context { 
            logger: logger,
            files: Vec::new(),
        };

        context
    }

    pub fn fsink(&mut self, dir: impl Into<PathBuf>, name: impl Into<String>) -> &mut Self {
        let _dir = dir.into();
        let _name = name.into();
        let _log = Logger::create_file_logger(_name.as_str(), _dir);
        self.logger.add_context(&_name, _log);
        self.files.insert(self.files.len(), _name);
        self
    }

    pub fn log_info<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        self.logger.log_info(&_msg);
        if !self.files.is_empty()
        {
            let _files = self.files.clone();
            self.logger.log_info_files(_files, _msg);
        }
        self
    }
}

#[derive(Clone)]
pub struct LoggerHandle {
    sender: Sender<String>,
    fsender: Sender<(Vec<String>, String)>,
}

impl LoggerHandle {
    pub fn log_info<S: Into<String>>(&self, msg: S) {
        // Don't actually do the logging here, who knows what thread invoked us!
        self.sender.send(msg.into()).ok();
    }

    pub fn send_context<S: Into<String>>(&self, files: Vec<String>, msg: S) {
        self.fsender.send((files, msg.into())).ok();
    }
}

pub struct Logger {
    output: slog::Logger,
    files: BTreeMap<String, slog::Logger>,
    incoming: Receiver<String>,
    fincoming: Receiver<(Vec<String>, String)>,

    aggregate_log: bool,
}

impl Logger {
    pub fn new() -> (LoggerHandle, Logger) {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let _out = slog::Logger::root(drain, o!());

        let (tx, rx) = channel::<String>();
        let (ftx, frx) = channel::<(Vec<String>, String)>();

        let logger = Logger {
            output: _out,
            files: BTreeMap::new(),
            incoming: rx,
            fincoming: frx,

            // Default.
            aggregate_log: false,
        };

        let logger_handle = LoggerHandle {
            sender: tx,
            fsender: ftx,
        };

        (logger_handle, logger)
    }

    pub fn all_log(&mut self, dir: PathBuf) -> &mut Self
    {
        match self.files.get("All") {
            Some(s) => (),
            None => {
                let logger = Logger::create_file_logger("All.txt", dir);
                self.files.insert(
                    "All".to_string(),
                    logger);
            }
        }

        self.aggregate_log = true;
        self
    }

    fn create_file_logger<'a>(name: &'a str, dir: PathBuf) -> slog::Logger {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(dir.join(&name))
            .unwrap();

        let decorator = slog_term::PlainDecorator::new(file);
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, o!());

        logger
    }

    pub fn poll_once(&self) {
        while let Ok(msg) = self.incoming.try_recv() {
            // log for real now, now that we're in the desired thread and environment
            self.log_info(msg);
        }
    }

    pub fn poll_files(&self) {
        while let Ok(msg) = self.fincoming.try_recv() {
            for file in msg.0 {
                self.log_info_file(file, &msg.1)
            }
        }
    }

    fn context(&mut self, name: impl Into<String>, log: slog::Logger) -> &mut Self {
        self.files.insert(name.into(), log);
        self
    }

    fn log_info<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _info!(self.output, "{}", _msg);
        if self.aggregate_log
        {
            _info!(self.files["All"], "{}", _msg);
        }
        self
    }

    fn log_info_file<F: Into<String>, S: Into<String>>(&self, file: F, msg: S) {
        let _msg = msg.into();
        let _file = file.into();
        _info!(self.files[_file.as_str()], "{}", _msg);
    }
}

pub struct Logging {
    handle: LoggerHandle,
    logger: Arc<RwLock<Logger>>
}

impl Logging {
    pub fn new(logh: LoggerHandle, log: Arc<RwLock<Logger>>) -> Logging {

        let logging = Logging {
            handle: logh,
            logger: log
        };

        logging
    }

    pub fn add_context(&self, name: impl Into<String>, log: slog::Logger) -> &Self {
        self.logger.try_write().unwrap().context(name.into(), log);

        self
    }

    pub fn log_info<S: Into<String>>(&self, msg: S) -> &Self {
        self.handle.log_info(msg);
        self.logger.try_read().unwrap().poll_once();

        self
    }

    pub fn log_info_files<S: Into<String>>(&self, files: Vec<String> ,msg: S) -> &Self {
        self.handle.send_context(files, msg);
        self.logger.try_read().unwrap().poll_files();

        self
    }
}

#[macro_export]
macro_rules! info {
    ($logging:expr, $($message:tt)*) => {
        $crate::Context::log_info($logging, format!($($message)*))
    }
}

/* FOR USE LATER

Ok, so Arc<Mutex> will ensure that only one reader or one writer can exist at a time.
Arc<RwLock> will allow any number of readers, or one writer at a time.
Also, only use Rc in a single-threaded context.

Cell, RefCell, and Box should also only be used in a single-threaded context.

If you always are going to clone, accept Arc. If you sometimes clone, pass &Arc. If you only need to read from the value, pass &


let (rx, tx) = Mpsc::channel();
tx.send((Vec::new::<String>(), "".into());

----------------------------

#[derive(Clone)]
struct LoggerHandle {
    sender: Sender<String>,
}

impl LoggerHandle {
    fn log_info<S: Into<String>>(&self, msg: S) {
        // Don't actually do the logging here, who knows what thread invoked us!
        self.sender.send(msg.into()).ok();
    }
}

-----------------------

pub struct Logger {
    output: slog::Logger,
    files: BTreeMap<String, slog::Logger>,
    incoming: Receiver<String>,
}

impl Logger {
    pub fn poll_once(&self) {
        while let Ok(msg) = self.incoming.try_recv() {
            // log for real now, now that we're in the desired thread and environment
            self.log_info(msg);
        }
    }
}


*/

/*
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

        let log = Logger::create_file_logger("All.txt", dir.into());

        logger.files.insert("All", log);

        logger
    }

    pub fn add_context(
        &mut self,
        name: Option<&'static str>,
        dir: impl Into<PathBuf>,
    ) -> &mut Self {
        let log = Logger::create_file_logger(name.unwrap(), dir.into());

        self.files.insert(name.unwrap(), log);

        self
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

    pub fn log_error<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _error!(self.output, "{}", _msg);
        _error!(self.files["All"], "{}", _msg);

        self
    }

    pub fn log_warn<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _warn!(self.output, "{}", _msg);
        _warn!(self.files["All"], "{}", _msg);

        self
    }

    pub fn log_info<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _info!(self.output, "{}", _msg);
        _info!(self.files["All"], "{}", _msg);

        self
    }

    pub fn log_debug<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _debug!(self.output, "{}", _msg);
        _debug!(self.files["All"], "{}", _msg);

        self
    }

    pub fn log_trace<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _trace!(self.output, "{}", _msg);
        _trace!(self.files["All"], "{}", _msg);

        self
    }
}

#[macro_export]
macro_rules! _format_args {
    ($($args:tt),*) => {
        //format_args!(($($args)*)).to_str()
        $crate::_format_args!(@ $($args),*)
    };

    (@ $first:tt, $(, $rem:tt)*) => {
        format_args!(($first), ($($rem:tt),*));
    }
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $($message:tt)*) => {
        $crate::Logger::log_error($logger, format!($($message)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($logger:expr, $($message:tt)*) => {
        $crate::Logger::log_warn($logger, format!($($message)*))
    }
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $($message:tt)*) => {
        $crate::Logger::log_info($logger, format!($($message)*))
    }
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $($message:tt)*) => {
        $crate::Logger::log_debug($logger, format!($($message)*))
    }
}

#[macro_export]
macro_rules! trace {
    ($logger:expr, $($message:tt)*) => {
        $crate::Logger::log_trace($logger, format!($($message)*))
    }
}
*/

/*
// This is something to play with later for greater control.
#[macro_export]
macro_rules! info {
    ($logger:expr, $($message:tt),*) => {
        //$crate::Logger::log_info($logger, $crate::_format_args!($($message)*))
        $crate::info!(@ $logger, $($message),*)
    };

    (@ $logger:expr, $first:tt $(, $rem:tt)*) => {
        $crate::Logger::log_info($logger, format!(($first), $($rem),*));
    };
}
*/
