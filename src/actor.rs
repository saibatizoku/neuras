//! Actors that interact over the network.
//!
//! `Actorling` entities live on the network. They are known for having a unique `address`
//! (e.g. `tcp://127.0.0.1:5784`, `ipc://var/sockets/socket_1234`, `udp://...`, etc.) on the
//! network, or on a local Unix-socket.
//!
//! An `Actorling` listens on this `æddress` for messages arriving over the network, sent by
//! other actors.
//!
//! Upon receiving a message, an `Actorling` can only do one of three things:
//!
//! * Add new actorlings to the network
//! * Send messages to other actors
//! * Define what to do with the next message
//!
//! # Example
//!
//! ```
//! extern crate neuras;
//! extern crate zmq;
//!
//! use neuras::actor::Actorling;
//!
//! fn main () {
//!     let actorling = Actorling::new("inproc://test_actor").unwrap();
//!     let _ = actorling.start().unwrap();
//!     let mut msg = zmq::Message::new();
//!     let pipe = actorling.pipe();
//!     {
//!         let _ = pipe.send("PING", 0).unwrap();
//!         let _ = pipe.recv(&mut msg, 0).unwrap();
//!     }
//!     let status = msg.as_str().unwrap();
//!
//!     println!("status: {}", &status);
//!     assert_eq!("PONG", status);
//!
//!     let _ = actorling.stop().unwrap();
//!
//!     ::std::process::exit(0);
//! }
//! ```
//!
pub mod errors {
    //! Actorling Errors.
    use std::io;
    use zmq;
    use socket;
    error_chain! {
        errors {
            AddressNotSpecified {
                description("the actorling needs to have an addess to work")
            }
            NotStarted {
                description("actorling is not started")
            }
            NotStopped {
                description("unable to stop actorling")
            }
        }
        links {
            Socket(socket::errors::Error, socket::errors::ErrorKind);
        }
        foreign_links {
            Io(io::Error);
            Zmq(zmq::Error);
        }
    }
}

use self::errors::*;

use std::thread;
use uuid::{Uuid, NAMESPACE_DNS};
use zmq;

#[allow(dead_code)]
/// A base type for actor-like entities
pub struct Actorling {
    address: String,
    context: zmq::Context,
    pipe: zmq::Socket,
    uuid: Uuid,
}

impl Actorling {
    /// Create a new `Actorling` instance with the address that it will be known for within
    /// the network.
    pub fn new(addr: &str) -> Result<Self> {
        Actorling::new_with_context(addr, zmq::Context::new())
    }

    /// Create a new `Actorling` instance that shares network context with the creator.
    /// Useful for creating actors in the same process (possibly/commonly in child threads),
    /// that can talk to the creator actor (usually running on the main thread, but could be
    /// run from a child thread as well).
    pub fn new_with_context(addr: &str, context: zmq::Context) -> Result<Self> {
        let address = addr.to_string();
        let pipe = context.socket(zmq::PAIR).unwrap();
        let uuid = Uuid::new_v5(&NAMESPACE_DNS, "actorling");
        let actorling = Actorling {
            address,
            context,
            pipe,
            uuid,
        };
        Ok(actorling)
    }
}

impl Actorling {
    /// Returns a `String` with the address for the Actorling.
    pub fn address(&self) -> String {
        self.address.clone()
    }

    /// Returns the actorling's network context.
    /// Useful for spawning sockets, and for creating sibling actors (see
    /// `Actorling::new_with_context`).
    pub fn context(&self) -> zmq::Context {
        self.context.clone()
    }

    /// Function for spawing child-threads, returning the `thread::JoinHandle`.
    pub fn run_thread<F, T>(&self, name: &str, callback: F) -> Result<thread::JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        run_thread(name, callback)
    }

    /// Start the current actorling instance.
    pub fn start(&self) -> Result<()> {
        // We create a new UUID that will only be known to each PAIR socket at runtime.
        let paddr = uuid_pipe_address();
        let _ = self.pipe.connect(&paddr)?;
        unimplemented!();
    }

    /// Stops the current actorling instance.
    pub fn stop(&self) -> Result<()> {
        unimplemented!();
    }

    /// Returns the actorling's UUID as a `String`
    pub fn uuid(&self) -> String {
        self.uuid.simple().to_string()
    }
}

/// Function for spawing child-threads, returning the `thread::JoinHandle`.
pub fn run_thread<F, T>(name: &str, callback: F) -> Result<thread::JoinHandle<T>>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let handler = thread::Builder::new()
        .name(name.to_string())
        .spawn(callback)
        .chain_err(|| "could not spawn actorling thread");
    handler
}

fn uuid_pipe_address() -> String {
    let uuid = Uuid::new_v5(&NAMESPACE_DNS, "actorling");
    format!("inproc://{}", uuid.simple().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actorlings_are_created_with_fn_new() {
        let acty = Actorling::new("inproc://my_actorling");
        assert!(acty.is_ok());
    }

    #[test]
    fn actorlings_return_ok_on_start() {
        let acty = Actorling::new("inproc://my_actorling").unwrap();
        let start = acty.start();
        assert!(start.is_ok());
        let _ = acty.stop().unwrap();
    }

    #[test]
    fn actorlings_return_ok_on_stop() {
        let acty = Actorling::new("inproc://my_actorling").unwrap();
        let _ = acty.start().unwrap();
        let stop = acty.stop();
        assert!(stop.is_ok());
    }

    #[test]
    fn actorlings_return_err_if_stopped_when_not_running() {
        let acty = Actorling::new("inproc://my_actorling").unwrap();
        let stop = acty.stop();
        assert!(stop.is_err());
    }
}
