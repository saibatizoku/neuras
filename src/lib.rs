#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
pub extern crate url;
pub extern crate zmq;

/// Error handling.
pub mod errors;
/// Usueful utilities to deal with ZMQ.
pub mod utils;
