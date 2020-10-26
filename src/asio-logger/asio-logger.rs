// {{{ Crate docs
//!
// }}}

// {{{ Imports & meta
//#![warn(missing_docs)]
// For development.
#![allow(unused_variables)]
#![allow(non_snake_case)]

use slog::{Drain, Record, OwnedKVList};
use std::collections::BTreeMap;

// {{{ Slog Manager

/// Describes potential errors when calling Context
/// TODO: elaborate/extend
#[derive(Debug)]
pub enum ContextError {
    /// Fatal problem
    Fatal(Box<dyn std::error::Error>),
}

/// Alias.
pub type ContextResult<T> = std::result::Result<T, ContextError>;

pub struct SlogManager<D>
    where
        D: slog::Drain<Err = slog::Never, Ok = ()> + Send + 'static,
{
    drain_map: BTreeMap<String, D>,
    all_drains: Vec<D>,
}

impl<D> SlogManager<D>
    where
        D: slog::Drain<Err = slog::Never, Ok = ()> + Send + 'static,
{
    pub fn new() -> SlogManager<D> {
        SlogManager {
            drain_map: BTreeMap::new(),
            all_drains: Vec::new(),
        }
    }

    pub fn add_all_drain(&mut self, drain: D) -> &Self {
        self.all_drains.push(drain);
        self
    }

    pub fn add(&mut self, name: impl Into<String>, drain: D) -> &Self {
        self.drain_map.insert(name.into(), drain);
        self
    }
}

pub struct Context<'a, D>
    where
        D: slog::Drain<Err = slog::Never, Ok = ()> + Send + 'static,
{
    asio: &'a SlogManager<D>,
    log_drains: Vec<String>,
}

impl<'a, D> Context<'a, D>
    where
        D: slog::Drain<Err = slog::Never, Ok = ()> + Send + 'static,
{
    pub fn new(slog_manager: &'a SlogManager<D>) -> Context<D> {
        Context {
            asio: slog_manager,
            log_drains: Vec::new(),
        }
    }

    pub fn add(&mut self, key: impl Into<String>) -> &Self {
        self.log_drains.push(key.into());
        self
    }
}

impl<D> Drain for Context<'_, D>
    where
        D: slog::Drain<Err = slog::Never, Ok = ()> + Send + 'static,
{
    type Ok = ();
    // TODO: Expand
    type Err = ContextError;

    // TODO: This is the expensive function,
    // TODO: will eventually be async.
    fn log(
        &self,
        record: &Record,
        logger_values: &OwnedKVList,
    ) -> ContextResult<()> {
        for all_drain in &self.asio.all_drains {
            match all_drain.log(record, logger_values) {
                Ok(()) => {}
                Err(ContextError) => {}
            }
        }
        for drain in &self.log_drains {
            match self.asio.drain_map.get(drain).unwrap().log(record, logger_values) {
                Ok(()) => {}
                Err(ContextError) => {}
            }
        }
        Ok(())
    }
}
