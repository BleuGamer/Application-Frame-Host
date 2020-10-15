// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

#[doc(hidden)]
pub use slog;

use slog::debug as _debug;
use slog::error as _error;
use slog::info as _info;
use slog::trace as _trace;
use slog::warn as _warn;

use slog::Level;
use slog::{o, Drain};
use std::collections::BTreeMap;
use std::fmt::write;
use std::fs::OpenOptions;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::sync::RwLock;

pub struct Context {
    logger: Arc<Logger>,
    files: Vec<String>,
}

impl Context {
    pub fn new(logger: Arc<Logger>, dir: impl Into<PathBuf>) -> Self {
        let context = Context {
            logger: logger,
            files: Vec::new(),
        };

        context
    }

    pub fn file(&mut self, dir: impl Into<PathBuf>, name: impl Into<String>) -> &mut Self {
        let _dir = dir.into();
        let _name = name.into();
        let _log = SlogManager::create_file_logger(_name.as_str(), _dir);
        self.logger.add_context(&_name, _log);
        self.files.insert(self.files.len(), _name);
        self
    }

    pub fn log_msg<S: Into<String>>(&self, level: slog::Level, msg: S) {
        let _msg = msg.into();
        self.logger.log_msg(level, &_msg);
        if !self.files.is_empty() {
            let _files = self.files.clone();
            self.logger.log_msg_files(level, _files, _msg);
        }
    }
}

#[derive(Clone)]
pub struct LoggerHandle {
    sender: Sender<(slog::Level, String)>,
    fsender: Sender<(slog::Level, Vec<String>, String)>,
}

impl LoggerHandle {
    pub fn log_msg<S: Into<String>>(&self, level: slog::Level, msg: S) {
        // Don't actually do the logging here, who knows what thread invoked us!
        self.sender.send((level, msg.into())).ok();
    }

    pub fn send_context<S: Into<String>>(&self, level: slog::Level, files: Vec<String>, msg: S) {
        self.fsender.send((level, files, msg.into())).ok();
    }
}

pub struct SlogManager {
    output: slog::Logger,
    files: BTreeMap<String, slog::Logger>,
    incoming: Receiver<(slog::Level, String)>,
    fincoming: Receiver<(slog::Level, Vec<String>, String)>,

    aggregate_log: bool,
}

impl SlogManager {
    pub fn new() -> (LoggerHandle, SlogManager) {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let _out = slog::Logger::root(drain, o!());

        let (tx, rx) = channel::<(slog::Level, String)>();
        let (ftx, frx) = channel::<(slog::Level, Vec<String>, String)>();

        let logger = SlogManager {
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

    pub fn all_log(&mut self, dir: PathBuf) -> &mut Self {
        match self.files.get("log_all.txt") {
            Some(s) => (),
            None => {
                let logger = SlogManager::create_file_logger("All.txt", dir);
                self.files.insert("log_all.txt".to_string(), logger);
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
            self.log_msg(msg.0, msg.1);
        }
    }

    pub fn poll_files(&self) {
        while let Ok(msg) = self.fincoming.try_recv() {
            for file in msg.1 {
                self.log_msg_file(msg.0, &file, &msg.2)
            }
        }
    }

    fn context(&mut self, name: impl Into<String>, log: slog::Logger) -> &mut Self {
        let _name = name.into();
        match self.files.get(_name.as_str()) {
            Some(s) => (),
            None => {
                self.files.insert(_name, log);
            }
        }
        self
    }

    fn log_msg<S: Into<String>>(&self, level: slog::Level, msg: S) -> &Self {
        let _msg = msg.into();

        match level {
            slog::Level::Error => {
                _error!(self.output, "{}", _msg);
                if self.aggregate_log {
                    _error!(self.files["log_all.txt"], "{}", _msg);
                }
            }
            slog::Level::Warning => {
                _warn!(self.output, "{}", _msg);
                if self.aggregate_log {
                    _warn!(self.files["log_all.txt"], "{}", _msg);
                }
            }
            slog::Level::Info => {
                _info!(self.output, "{}", _msg);
                if self.aggregate_log {
                    _info!(self.files["log_all.txt"], "{}", _msg);
                }
            }
            slog::Level::Debug => {
                _debug!(self.output, "{}", _msg);
                if self.aggregate_log {
                    _debug!(self.files["log_all.txt"], "{}", _msg);
                }
            }
            slog::Level::Trace => {
                _trace!(self.output, "{}", _msg);
                if self.aggregate_log {
                    _trace!(self.files["log_all.txt"], "{}", _msg);
                }
            }
            _ => ()
        }

        self
    }

    fn log_msg_file<S: Into<String>>(&self, level: slog::Level, file: S, msg: S) {
        let _msg = msg.into();
        let _file = file.into();

        match level {
            slog::Level::Error => {
                _error!(self.files[_file.as_str()], "{}", _msg);
            }
            slog::Level::Warning => {
                _warn!(self.files[_file.as_str()], "{}", _msg);
            }
            slog::Level::Info => {
                _info!(self.files[_file.as_str()], "{}", _msg);
            }
            slog::Level::Debug => {
                _debug!(self.files[_file.as_str()], "{}", _msg);
            }
            slog::Level::Trace => {
                _trace!(self.files[_file.as_str()], "{}", _msg);
            }
            _ => ()
        }
    }
}

pub struct Logger {
    handle: LoggerHandle,
    logger: Arc<RwLock<SlogManager>>,
}

impl Logger {
    pub fn new(logh: LoggerHandle, log: Arc<RwLock<SlogManager>>) -> Logger {
        let logging = Logger {
            handle: logh,
            logger: log,
        };

        logging
    }

    pub fn add_context(&self, name: impl Into<String>, log: slog::Logger) -> &Self {
        self.logger.try_write().unwrap().context(name.into(), log);

        self
    }

    pub fn log_msg<S: Into<String>>(&self, level: slog::Level, msg: S) -> &Self {
        self.handle.log_msg(level, msg.into());
        self.logger.try_read().unwrap().poll_once();

        self
    }

    pub fn log_msg_files<S: Into<String>>(&self, level: slog::Level, files: Vec<String>, msg: S) -> &Self {
        self.handle.send_context(level.into(), files, msg.into());
        self.logger.try_read().unwrap().poll_files();

        self
    }
}

#[macro_export]
macro_rules! error {
    ($logging:expr, $($message:tt)*) => {
        $crate::Context::log_msg($logging, $crate::slog::Level::Error, format!($($message)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($logging:expr, $($message:tt)*) => {
        $crate::Context::log_msg($logging, $crate::slog::Level::Warning, format!($($message)*))
    }
}

#[macro_export]
macro_rules! info {
    ($logging:expr, $($message:tt)*) => {
        $crate::Context::log_msg($logging, $crate::slog::Level::Info, format!($($message)*))
    }
}

#[macro_export]
macro_rules! debug {
    ($logging:expr, $($message:tt)*) => {
        $crate::Context::log_msg($logging, $crate::slog::Level::Debug, format!($($message)*))
    }
}

#[macro_export]
macro_rules! trace {
    ($logging:expr, $($message:tt)*) => {
        $crate::Context::log_msg($logging, $crate::slog::Level::Trace, format!($($message)*))
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

        logger.files.insert("log_all.txt", log);

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
        _error!(self.files["log_all.txt"], "{}", _msg);

        self
    }

    pub fn log_warn<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _warn!(self.output, "{}", _msg);
        _warn!(self.files["log_all.txt"], "{}", _msg);

        self
    }

    pub fn log_info<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _info!(self.output, "{}", _msg);
        _info!(self.files["log_all.txt"], "{}", _msg);

        self
    }

    pub fn log_debug<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _debug!(self.output, "{}", _msg);
        _debug!(self.files["log_all.txt"], "{}", _msg);

        self
    }

    pub fn log_trace<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _trace!(self.output, "{}", _msg);
        _trace!(self.files["log_all.txt"], "{}", _msg);

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
