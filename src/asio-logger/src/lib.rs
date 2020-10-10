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
use std::sync::mpsc::{Receiver, Sender};

pub struct Context<'a> {
    logger: &'a Logger,
}

impl<'a> Context<'a> {
    pub fn new(logger: &'a Logger, dir: impl Into<PathBuf>) -> Self {
        let mut context = Context { logger: logger };

        let log = Context::create_file_logger("All.txt", dir.into());
        logger.add_context("All", log);

        context
    }

    pub fn fsink(&mut self, dir: impl Into<PathBuf>, name: impl Into<String>) -> &mut Self {
        let _dir = dir.into();
        let _name = name.into();
        let _log = Context::create_file_logger(_name.as_str(), _dir);
        self.logger.add_context(_name, _log);
        self
    }

    fn create_file_logger<'b>(name: &'b str, dir: PathBuf) -> slog::Logger {
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
}

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

pub struct Logger {
    output: slog::Logger,
    files: BTreeMap<String, slog::Logger>,
}

impl Logger {
    pub fn new(messenger: &T) -> Logger<T> {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let _out = slog::Logger::root(drain, o!());

        let logger = Logger {
            output: _out,
            files: BTreeMap::new(),
        };

        logger
    }

    fn add_context(&self, name: impl Into<String>, log: slog::Logger) -> &Self {
        self.files.insert(name.into(), log);
        self
    }

    fn log_info<S: Into<String>>(&self, msg: S) -> &Self {
        let _msg = msg.into();
        _info!(self.output, "{}", _msg);

        self
    }
}

/* FOR USE LATER

Ok, so Arc<Mutex> will ensure that only one reader or one writer can exist at a time.
Arc<RwLock> will allow any number of readers, or one writer at a time.
Also, only use Rc in a single-threaded context.

Cell, RefCell, and Box should also only be used in a single-threaded context.

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

    pub fn add_context(&mut self, name: Option<&'static str>, dir: impl Into<PathBuf>) -> &mut Self {

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
