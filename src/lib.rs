//! neuras - A high-level API for networking with ØMQ (zeromq)
//! ==========================================================
//!
//! An attempt at having a high-level API on top of ØMQ's awesome foundations,
//! as suggested by
//! "[Features of a Higher-Level API](http://zguide.zeromq.org/page:all#toc74)",
//! by using tokio's reactor and tools.
#![recursion_limit = "1024"]

#[macro_use]
extern crate bitflags;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate slab;
extern crate toml;
extern crate url;
extern crate uuid;

extern crate mio as mio_lib;
extern crate zmq;

// Optional crates from `async-tokio` feature
#[cfg(feature = "async-tokio")]
extern crate futures;
#[cfg(feature = "async-tokio")]
extern crate tokio_core;
#[cfg(feature = "async-tokio")]
extern crate tokio_signal;

// Actors that interact over the network.
pub mod actor;
// Millisecond clocks and delays.
pub mod clock;
// Library initialization scheme.
mod initialize;
// Messages for sockets.
mod message;
// Polling for sockets.
pub mod poller;
// Proxy actor.
mod proxy;
// Sockets for networking.
pub mod socket;
// Useful utilities to deal with ZMQ.
pub mod utils;

// Convenient API type for dealing with clocks and delays.
pub use clock::Clock;
pub use socket::Socket;
