#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate url;
extern crate zmq;

/// Error handling.
pub mod errors;
/// Usueful utilities to deal with ZMQ.
pub mod utils;

pub use utils::{create_context, create_message, subscribe_client, zmq_xpub_xsub_proxy, zmq_xpub, zmq_xsub, zmq_pub, zmq_sub, zmq_rep, zmq_req, bind_server, connect_client};
