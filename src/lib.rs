#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate url;
extern crate zmq;

/// Error handling.
pub mod errors;
/// Node entities for networking.
pub mod node;
/// Usueful utilities to deal with ZMQ.
pub mod utils;

pub use zmq::{Context, CurveKeyPair, Message, Socket};
