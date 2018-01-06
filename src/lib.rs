//! neuras - A high-level API for networking with ØMQ (zeromq)
//! ==========================================================
//!
//! An attempt at having a high-level API on top of ØMQ's awesome foundations,
//! as suggested by
//! "[Features of a Higher-Level API](http://zguide.zeromq.org/page:all#toc74)",
//! by using tokio's reactor and tools.
//!
//!
//! ```
//! extern crate neuras;
//!
//! use neuras::{init, Socket};
//!
//! const PAIR_ENDPOINT: &str = "inproc://push";
//!
//! fn main () {
//!     /// RUN THIS ALWAYS FIRST AND ON THE MAIN THREAD.
//!     /// If you don't, beware.... there be monsters here.
//!     let _ = neuras::init();
//!
//!     let push = Socket::new_push(PAIR_ENDPOINT).unwrap();
//!
//!     let pull = Socket::new_pull(PAIR_ENDPOINT).unwrap();
//!
//!     // let _ = push.send("hi").unwrap();
//!     // let msg_from_push = pull.recv().unwrap();
//!     // println!("{}", &msg_from_push);
//!
//!     // let _ = pull.send("hi").unwrap();
//!     // let msg_from_pull = push.recv().unwrap();
//!     // println!("{}", &msg_from_pull);
//! }
//! ```

#![recursion_limit = "1024"]

#[macro_use]
extern crate bitflags;
extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate futures;
#[macro_use]
extern crate serde_derive;
extern crate tokio_core;
extern crate tokio_signal;
extern crate toml;
extern crate url;
extern crate uuid;
extern crate zmq;
extern crate zmq_tokio;

// Actors that interact over the network.
pub mod actor;
// Millisecond clocks and delays.
pub mod clock;
// Crate-wide error chain.
pub mod errors;
// Library initialization scheme.
mod initialize;
// Messages for sockets.
pub mod message;
// Polling for sockets.
pub mod poller;
// Proxy actor.
pub mod proxy;
// Security for socket communications.
pub mod security;
// Sockets for networking.
pub mod socket;
// Useful utilities to deal with ZMQ.
pub mod utils;

pub use initialize::init;
// Convenient API type for dealing with clocks and delays.
pub use clock::Clock;
pub use socket::Socket;
