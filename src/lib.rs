//! neuras - A high-level API for networking with ØMQ ("zeromq") and tokio.
//! =======================================================================
//!
//! An attempt at having a high-level API on top of ØMQ's awesome foundations,
//! as suggested by
//! "[Features of a Higher-Level API](http://zguide.zeromq.org/page:all#toc74)",
//! by using tokio's reactor and tools.
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
// Secure-socket communications.
pub mod secure;
// Useful utilities to deal with ZMQ.
pub mod utils;
