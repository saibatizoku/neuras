#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
pub extern crate futures;
pub extern crate tokio_core;
pub extern crate url;
pub extern crate zmq;
pub extern crate zmq_tokio;

/// Error handling.
pub mod errors;
/// Secure-socket communications.
pub mod secure;
/// Useful utilities to deal with ZMQ.
pub mod utils;
