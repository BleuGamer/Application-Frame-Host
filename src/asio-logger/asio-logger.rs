// {{{ Crate docs
//!
// }}}

// {{{ Imports & meta
#![warn(missing_docs)]
// For development.
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate slog;
extern crate crossbeam_channel;
extern crate take_mut;
extern crate thread_local;

use crossbeam_channel::Sender;

use slog::{BorrowedKV, Level, Record, RecordStatic, SingleKV, KV};

use slog::{Key, OwnedKVList, Serializer};

use slog::Drain;
use std::error::Error;
use std::fmt;
use std::sync;
use std::{io, thread};

use once_cell::sync::OnceCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use take_mut::take;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::ops::{Deref, DerefMut};
// }}}

// {{{ Log Manager
/// Only one instance of SlogManager can ever exist.
static SM_INSTANCE: OnceCell<Mutex<SlogManager>> = OnceCell::new();

struct SlogManager {
    drains: Vec<Box<dyn Drain<Err = slog::Never, Ok = ()> + Send + 'static>>,
}

impl SlogManager {
    fn init_or_get() -> &'static Mutex<SlogManager> {
        let test_set = SM_INSTANCE.get();
        return match test_set {
            Some(t) => test_set.unwrap(),
            _ => {
                let sm = SlogManager {
                    drains: Vec::new(),
                };
                let err = SM_INSTANCE.set(Mutex::new(sm));
                match err {
                    Ok(r) => (),
                    Err(e) => panic!("You should never see this message.\
                                                  Something went very, very, wrong.")
                }
                test_set.unwrap()
            }
        }
    }

    fn insert(&mut self, drain: Box<dyn Drain<Err = slog::Never, Ok = ()> + Send + 'static>) {
        self.drains.push(drain);
    }
}

/*
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

    pub fn slogger(
        self,
        dir: impl Into<PathBuf>,
        name: impl Into<String>,
        slogger: slog::Logger,
    ) -> Context {
        let _dir = dir.into();
        let _name = name.into();
        let _log = SlogManager::create_explicit_logger(_name.as_str(), _dir, slogger);
        self.logger.add_context(&_name, _log);
        let mut _files = self.files;
        _files.insert(_files.len(), _name);

        Context {
            logger: self.logger,
            files: _files,
        }
    }

    pub fn get<T: Into<String>>(&self, slogger: T) -> &slog::Logger {
        let slogger = self.logger.get_context(slogger.into());
        slogger
    }

    pub fn ref_slogger(
        &self,
        dir: impl Into<PathBuf>,
        name: impl Into<String>,
        slogger: slog::Logger,
    ) -> Context {
        let _dir = dir.into();
        let _name = name.into();
        let _log = SlogManager::create_explicit_logger(_name.as_str(), _dir, slogger);
        self.logger.add_context(&_name, _log);
        let mut _files = self.files.to_owned();
        _files.insert(self.files.len(), _name);

        Context {
            logger: self.logger.to_owned(),
            files: _files.to_owned(),
        }
    }

    pub fn to_owned(&mut self) -> Context {
        Context {
            logger: self.logger.to_owned(),
            files: self.files.to_owned(),
        }
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
    fn init()  {
        let set = SM_INSTANCE.get();
        if set.unwrap() != *false {
            panic!("Only one instance of SlogManager can be in memory!");
        }

        SM_INSTANCE.set(true);
    }

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

    fn create_explicit_logger<'a>(
        name: &'a str,
        dir: PathBuf,
        slogger: slog::Logger,
    ) -> slog::Logger {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(dir.join(&name))
            .unwrap();

        let decorator = slog_term::PlainDecorator::new(file);
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, o!(slogger.list().to_owned()));

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
            _ => (),
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
            _ => (),
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

    pub fn get_context(&self, name: impl Into<String>) -> &slog::Logger {
        let slogger = self
            .logger
            .try_read()
            .unwrap()
            .files
            .get(&name.into())
            .unwrap();
        slogger
    }

    pub fn log_msg<S: Into<String>>(&self, level: slog::Level, msg: S) -> &Self {
        self.handle.log_msg(level, msg.into());
        self.logger.try_read().unwrap().poll_once();

        self
    }

    pub fn log_msg_files<S: Into<String>>(
        &self,
        level: slog::Level,
        files: Vec<String>,
        msg: S,
    ) -> &Self {
        self.handle.send_context(level.into(), files, msg.into());
        self.logger.try_read().unwrap().poll_files();

        self
    }
}
*/

/*
#[macro_export]
macro_rules! error {
    ($logging:expr, $($message:tt)*) => {
        $crate::context::Context::log_msg($logging, $crate::slog::Level::Error, format!($($message)*))
    }
}

#[macro_export]
macro_rules! warn {
    ($logging:expr, $($message:tt)*) => {
        $crate::context::Context::log_msg($logging, $crate::slog::Level::Warning, format!($($message)*))
    }
}

#[macro_export]
macro_rules! info {
    ($logging:expr, $($message:tt)*) => {
        $crate::context::Context::log_msg($logging, $crate::slog::Level::Info, format!($($message)*))
    }
}

#[macro_export]
macro_rules! debug {
    ($logging:expr, $($message:tt)*) => {
        $crate::context::Context::log_msg($logging, $crate::slog::Level::Debug, format!($($message)*))
    }
}

#[macro_export]
macro_rules! trace {
    ($logging:expr, $($message:tt)*) => {
        $crate::context::Context::log_msg($logging, $crate::slog::Level::Trace, format!($($message)*))
    }
}
*/

// }}}

// {{{ Serializer
struct ToSendSerializer {
    kv: Box<dyn KV + Send>,
}

impl ToSendSerializer {
    fn new() -> Self {
        ToSendSerializer { kv: Box::new(()) }
    }

    fn finish(self) -> Box<dyn KV + Send> {
        self.kv
    }
}

impl Serializer for ToSendSerializer {
    fn emit_bool(&mut self, key: Key, val: bool) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_unit(&mut self, key: Key) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, ()))));
        Ok(())
    }
    fn emit_none(&mut self, key: Key) -> slog::Result {
        let val: Option<()> = None;
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_char(&mut self, key: Key, val: char) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_u8(&mut self, key: Key, val: u8) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_i8(&mut self, key: Key, val: i8) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_u16(&mut self, key: Key, val: u16) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_i16(&mut self, key: Key, val: i16) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_u32(&mut self, key: Key, val: u32) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_i32(&mut self, key: Key, val: i32) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_f32(&mut self, key: Key, val: f32) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_u64(&mut self, key: Key, val: u64) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_i64(&mut self, key: Key, val: i64) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_f64(&mut self, key: Key, val: f64) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_usize(&mut self, key: Key, val: usize) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_isize(&mut self, key: Key, val: isize) -> slog::Result {
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_str(&mut self, key: Key, val: &str) -> slog::Result {
        let val = val.to_owned();
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
    fn emit_arguments(&mut self, key: Key, val: &fmt::Arguments) -> slog::Result {
        let val = fmt::format(*val);
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }

    #[cfg(feature = "nested-values")]
    fn emit_serde(&mut self, key: Key, value: &slog::SerdeValue) -> slog::Result {
        let val = value.to_sendable();
        take(&mut self.kv, |kv| Box::new((kv, SingleKV(key, val))));
        Ok(())
    }
}
// }}}

// {{{ Async
// {{{ AsyncError
/// Errors reported by `Async`
#[derive(Debug)]
pub enum AsyncError {
    /// Could not send record to worker thread due to full queue
    Full,
    /// Fatal problem - mutex or channel poisoning issue
    Fatal(Box<dyn std::error::Error>),
}

impl<T> From<crossbeam_channel::TrySendError<T>> for AsyncError {
    fn from(_: crossbeam_channel::TrySendError<T>) -> AsyncError {
        AsyncError::Full
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for AsyncError {
    fn from(_: crossbeam_channel::SendError<T>) -> AsyncError {
        AsyncError::Fatal(Box::new(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "The logger thread terminated",
        )))
    }
}

impl<T> From<std::sync::PoisonError<T>> for AsyncError {
    fn from(err: std::sync::PoisonError<T>) -> AsyncError {
        AsyncError::Fatal(Box::new(io::Error::new(
            io::ErrorKind::BrokenPipe,
            err.to_string(),
        )))
    }
}

/// `AsyncResult` alias
pub type AsyncResult<T> = std::result::Result<T, AsyncError>;

// }}}

// {{{ AsyncCore
/// `AsyncCore` builder
pub struct AsyncCoreBuilder {
    chan_size: usize,
    blocking: bool,
    thread_name: Option<String>,
}

impl AsyncCoreBuilder {
    fn new(drain: Box<dyn Drain<Err = slog::Never, Ok = ()> + Send + 'static>) -> Self {
        SlogManager::init_or_get().lock().unwrap().deref_mut().insert(drain);
        AsyncCoreBuilder {
            chan_size: 128,
            blocking: false,
            thread_name: None,
        }
    }

    /// Configure a name to be used for the background thread.
    ///
    /// The name must not contain '\0'.
    ///
    /// # Panics
    ///
    /// If a name with '\0' is passed.
    pub fn thread_name(mut self, name: String) -> Self {
        assert!(name.find('\0').is_none(), "Name with \\'0\\' in it passed");
        self.thread_name = Some(name);
        self
    }

    /// Set channel size used to send logging records to worker thread. When
    /// buffer is full `AsyncCore` will start returning `AsyncError::Full` or block, depending on
    /// the `blocking` configuration.
    pub fn chan_size(mut self, s: usize) -> Self {
        self.chan_size = s;
        self
    }

    /// Should the logging call be blocking if the channel is full?
    ///
    /// Default is false, in which case it'll return `AsyncError::Full`.
    pub fn blocking(mut self, blocking: bool) -> Self {
        self.blocking = blocking;
        self
    }

    fn spawn_thread(self) -> (thread::JoinHandle<()>, Sender<AsyncMsg>) {
        let (tx, rx) = crossbeam_channel::bounded(self.chan_size);
        let mut builder = thread::Builder::new();
        if let Some(thread_name) = self.thread_name {
            builder = builder.name(thread_name);
        }
        let join = builder
            .spawn(move || loop {
                match rx.recv().unwrap() {
                    AsyncMsg::Record(r) => {
                        let rs = RecordStatic {
                            location: &*r.location,
                            level: r.level,
                            tag: &r.tag,
                        };

                        for drain in &SlogManager::init_or_get().lock().unwrap().deref_mut().drains {
                            drain.log(
                                &Record::new(&rs, &format_args!("{}", r.msg), BorrowedKV(&r.kv)),
                                &r.logger_values,
                            )
                                .unwrap();
                        }
                    }
                    AsyncMsg::Finish => return,
                }
            })
            .unwrap();

        (join, tx)
    }

    /// Build `AsyncCore`
    pub fn build(self) -> AsyncCore {
        self.build_no_guard()
    }

    /// Build `AsyncCore`
    pub fn build_no_guard(self) -> AsyncCore {
        let blocking = self.blocking;
        let (join, tx) = self.spawn_thread();

        AsyncCore {
            ref_sender: tx,
            tl_sender: thread_local::ThreadLocal::new(),
            join: Mutex::new(Some(join)),
            blocking,
        }
    }

    /// Build `AsyncCore` with `AsyncGuard`
    ///
    /// See `AsyncGuard` for more information.
    pub fn build_with_guard(self) -> (AsyncCore, AsyncGuard) {
        let blocking = self.blocking;
        let (join, tx) = self.spawn_thread();

        (
            AsyncCore {
                ref_sender: tx.clone(),
                tl_sender: thread_local::ThreadLocal::new(),
                join: Mutex::new(None),
                blocking,
            },
            AsyncGuard {
                join: Some(join),
                tx,
            },
        )
    }
}

/// Async guard
///
/// All `Drain`s are reference-counted by every `Logger` that uses them.
/// `Async` drain runs a worker thread and sends a termination (and flushing)
/// message only when being `drop`ed. Because of that it's actually
/// quite easy to have a left-over reference to a `Async` drain, when
/// terminating: especially on `panic`s or similar unwinding event. Typically
/// it's caused be a leftover reference like `Logger` in thread-local variable,
/// global variable, or a thread that is not being joined on. It might be a
/// program bug, but logging should work reliably especially in case of bugs.
///
/// `AsyncGuard` is a remedy: it will send a flush and termination message to
/// a `Async` worker thread, and wait for it to finish on it's own `drop`. Using it
/// is a simplest way to guarantee log flushing when using `slog_async`.
pub struct AsyncGuard {
    // Should always be `Some`. `None` only
    // after `drop`
    join: Option<thread::JoinHandle<()>>,
    tx: Sender<AsyncMsg>,
}

impl Drop for AsyncGuard {
    fn drop(&mut self) {
        let _err: Result<(), Box<dyn std::error::Error>> = {
            || {
                let _ = self.tx.send(AsyncMsg::Finish);
                let join = self.join.take().unwrap();
                if join.thread().id() != thread::current().id() {
                    // See AsyncCore::drop for rationale of this branch.
                    join.join().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "Logging thread worker join error",
                        )
                    })?;
                }
                Ok(())
            }
        }();
    }
}

/// Core of `Async` drain
///
/// See `Async` for documentation.
///
/// Wrapping `AsyncCore` allows implementing custom overflow (and other errors)
/// handling strategy.
///
/// Note: On drop `AsyncCore` waits for it's worker-thread to finish (after
/// handling all previous `Record`s sent to it). If you can't tolerate the
/// delay, make sure you drop it eg. in another thread.
pub struct AsyncCore {
    ref_sender: Sender<AsyncMsg>,
    tl_sender: thread_local::ThreadLocal<Sender<AsyncMsg>>,
    join: Mutex<Option<thread::JoinHandle<()>>>,
    blocking: bool,
}

impl AsyncCore {
    /// New `AsyncCore` with default parameters
    pub fn new(drain: Box<dyn Drain<Err = slog::Never,
                Ok = ()> + Send + 'static + std::panic::RefUnwindSafe>) -> Self
    {
        AsyncCoreBuilder::new(drain).build()
    }

    /// Build `AsyncCore` drain with custom parameters
    pub fn custom (
        drain: Box<dyn Drain<Err = slog::Never, Ok = ()> + Send + 'static>,
    ) -> AsyncCoreBuilder {
        AsyncCoreBuilder::new(drain)
    }
    fn get_sender(
        &self,
    ) -> Result<
        &crossbeam_channel::Sender<AsyncMsg>,
        std::sync::PoisonError<sync::MutexGuard<crossbeam_channel::Sender<AsyncMsg>>>,
    > {
        self.tl_sender.get_or_try(|| Ok(self.ref_sender.clone()))
    }

    /// Send `AsyncRecord` to a worker thread.
    fn send(&self, r: AsyncRecord) -> AsyncResult<()> {
        let sender = self.get_sender()?;

        if self.blocking {
            sender.send(AsyncMsg::Record(r))?;
        } else {
            sender.try_send(AsyncMsg::Record(r))?;
        }

        Ok(())
    }
}

impl Drain for AsyncCore {
    type Ok = ();
    type Err = AsyncError;

    fn log(&self, record: &Record, logger_values: &OwnedKVList) -> AsyncResult<()> {
        let mut ser = ToSendSerializer::new();
        record
            .kv()
            .serialize(record, &mut ser)
            .expect("`ToSendSerializer` can't fail");

        self.send(AsyncRecord {
            msg: fmt::format(*record.msg()),
            level: record.level(),
            location: Box::new(*record.location()),
            tag: String::from(record.tag()),
            logger_values: logger_values.clone(),
            kv: ser.finish(),
        })
    }
}

struct AsyncRecord {
    msg: String,
    level: Level,
    location: Box<slog::RecordLocation>,
    tag: String,
    logger_values: OwnedKVList,
    kv: Box<dyn KV + Send>,
}

enum AsyncMsg {
    Record(AsyncRecord),
    Finish,
}

impl Drop for AsyncCore {
    fn drop(&mut self) {
        let _err: Result<(), Box<dyn std::error::Error>> = {
            || {
                if let Some(join) = self.join.lock()?.take() {
                    let _ = self.get_sender()?.send(AsyncMsg::Finish);
                    if join.thread().id() != thread::current().id() {
                        // A custom Drain::log implementation could dynamically
                        // swap out the logger which eventually invokes
                        // AsyncCore::drop in the worker thread.
                        // If we drop the AsyncCore inside the logger thread,
                        // this join() either panic or dead-lock.
                        // TODO: Figure out whether skipping join() instead of
                        // panicking is desirable.
                        join.join().map_err(|_| {
                            io::Error::new(
                                io::ErrorKind::BrokenPipe,
                                "Logging thread worker join error",
                            )
                        })?;
                    }
                }
                Ok(())
            }
        }();
    }
}
// }}}

/// Behavior used when the channel is full.
///
/// # Note
///
/// More variants may be added in the future, without considering it a breaking change.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum OverflowStrategy {
    /// The message gets dropped and a message with number of dropped is produced once there's
    /// space.
    ///
    /// This is the default.
    ///
    /// Note that the message with number of dropped messages takes one slot in the channel as
    /// well.
    DropAndReport,
    /// The message gets dropped silently.
    Drop,
    /// The caller is blocked until there's enough space.
    Block,
    #[doc(hidden)]
    DoNotMatchAgainstThisAndReadTheDocs,
}

/// `Async` builder
pub struct AsyncBuilder {
    core: AsyncCoreBuilder,
    // Increment a counter whenever a message is dropped due to not fitting inside the channel.
    inc_dropped: bool,
}

impl AsyncBuilder {
    fn new(drain: Box<dyn Drain<Err = slog::Never, Ok = ()> + Send + 'static>) -> AsyncBuilder {
        AsyncBuilder {
            core: AsyncCoreBuilder::new(drain),
            inc_dropped: true,
        }
    }

    /// Set channel size used to send logging records to worker thread. When
    /// buffer is full `AsyncCore` will start returning `AsyncError::Full`.
    pub fn chan_size(self, s: usize) -> Self {
        AsyncBuilder {
            core: self.core.chan_size(s),
            ..self
        }
    }

    /// Sets what will happen if the channel is full.
    pub fn overflow_strategy(self, overflow_strategy: OverflowStrategy) -> Self {
        let (block, inc) = match overflow_strategy {
            OverflowStrategy::Block => (true, false),
            OverflowStrategy::Drop => (false, false),
            OverflowStrategy::DropAndReport => (false, true),
            OverflowStrategy::DoNotMatchAgainstThisAndReadTheDocs => panic!("Invalid variant"),
        };
        AsyncBuilder {
            core: self.core.blocking(block),
            inc_dropped: inc,
        }
    }

    /// Configure a name to be used for the background thread.
    ///
    /// The name must not contain '\0'.
    ///
    /// # Panics
    ///
    /// If a name with '\0' is passed.
    pub fn thread_name(self, name: String) -> Self {
        AsyncBuilder {
            core: self.core.thread_name(name),
            ..self
        }
    }

    /// Complete building `Async`
    pub fn build(self) -> Async {
        Async {
            core: self.core.build_no_guard(),
            dropped: AtomicUsize::new(0),
            inc_dropped: self.inc_dropped,
        }
    }

    /// Complete building `Async`
    pub fn build_no_guard(self) -> Async {
        Async {
            core: self.core.build_no_guard(),
            dropped: AtomicUsize::new(0),
            inc_dropped: self.inc_dropped,
        }
    }

    /// Complete building `Async` with `AsyncGuard`
    ///
    /// See `AsyncGuard` for more information.
    pub fn build_with_guard(self) -> (Async, AsyncGuard) {
        let (core, guard) = self.core.build_with_guard();
        (
            Async {
                core,
                dropped: AtomicUsize::new(0),
                inc_dropped: self.inc_dropped,
            },
            guard,
        )
    }
}

/// Async drain
///
/// `Async` will send all the logging records to a wrapped drain running in
/// another thread.
///
/// `Async` never returns `AsyncError::Full`.
///
/// `Record`s are passed to the worker thread through a channel with a bounded
/// size (see `AsyncBuilder::chan_size`). On channel overflow `Async` will
/// start dropping `Record`s and log a message informing about it after
/// sending more `Record`s is possible again. The exact details of handling
/// overflow is implementation defined, might change and should not be relied
/// on, other than message won't be dropped as long as channel does not
/// overflow.
///
/// Any messages reported by `Async` will contain `slog-async` logging `Record`
/// tag to allow easy custom handling.
///
/// Note: On drop `Async` waits for it's worker-thread to finish (after handling
/// all previous `Record`s sent to it). If you can't tolerate the delay, make
/// sure you drop it eg. in another thread.
pub struct Async {
    core: AsyncCore,
    dropped: AtomicUsize,
    // Increment the dropped counter if dropped?
    inc_dropped: bool,
}

impl Async {
    /// New `AsyncCore` with default parameters
    pub fn default(drain: Box<dyn Drain<Err = slog::Never, Ok = ()> + Send + 'static>) -> Self {
        AsyncBuilder::new(drain).build()
    }

    /// Build `Async` drain with custom parameters
    ///
    /// The wrapped drain must handle all results (`Drain<Ok=(),Error=Never>`)
    /// since there's no way to return it back. See `slog::DrainExt::fuse()` and
    /// `slog::DrainExt::ignore_res()` for typical error handling strategies.
    pub fn new(
        drain: Box<dyn Drain<Err = slog::Never, Ok = ()> + Send + 'static>,

    ) -> AsyncBuilder {
        AsyncBuilder::new(drain)
    }

    fn push_dropped(&self, logger_values: &OwnedKVList) -> AsyncResult<()> {
        let dropped = self.dropped.swap(0, Ordering::Relaxed);
        if dropped > 0 {
            match self.core.log(
                &record!(
                    slog::Level::Error,
                    "slog-async",
                    &format_args!(
                        "slog-async: logger dropped messages \
                             due to channel \
                             overflow"
                    ),
                    b!("count" => dropped)
                ),
                logger_values,
            ) {
                Ok(()) => {}
                Err(AsyncError::Full) => {
                    self.dropped.fetch_add(dropped + 1, Ordering::Relaxed);
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl Drain for Async {
    type Ok = ();
    type Err = AsyncError;

    // TODO: Review `Ordering::Relaxed`
    fn log(&self, record: &Record, logger_values: &OwnedKVList) -> AsyncResult<()> {
        self.push_dropped(logger_values)?;

        match self.core.log(record, logger_values) {
            Ok(()) => {}
            Err(AsyncError::Full) if self.inc_dropped => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            Err(AsyncError::Full) => {}
            Err(e) => return Err(e),
        }

        Ok(())
    }
}

impl Drop for Async {
    fn drop(&mut self) {
        let _ = self.push_dropped(&o!().into());
    }
}

// }}}
