//! [WIP] Actors that interact over the network.
//!
//! ```
//! extern crate neuras;
//! extern crate zmq;
//!
//! use std::thread;
//!
//! use neuras::{init, Socket};
//! use neuras::actor::Actorling;
//! use neuras::socket::socket_new_pair;
//! use neuras::errors::*;
//!
//! fn main () {
//!     let actorling = Actorling::new("inproc://test_actor");
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

use std::thread;

use futures::{Future, Sink, Stream};
use tokio_core::reactor::Handle;
use uuid::{Uuid, NAMESPACE_DNS};
use zmq;

use super::socket::Socket;
use self::errors::*;

#[allow(dead_code)]
/// A base type for actor-like entities
pub struct Actorling {
    address: String,
    context: zmq::Context,
    pipe: Socket,
    uuid: Uuid,
}

impl Actorling {
    /// Create a new `Actorling` instance with the address that it will be known for within
    /// the network.
    pub fn new(addr: &str) -> Result<Self> {
        let context = zmq::Context::new();
        let pipe = Socket::new(zmq::PAIR).unwrap();
        let uuid = Uuid::new_v5(&NAMESPACE_DNS, "actorling");
        let _ = pipe.bind(&uuid_pipe_address(&uuid))?;
        let actorling = Actorling {
            address: addr.to_string(),
            context,
            pipe,
            uuid,
        };
        Ok(actorling)
    }

    /// Create a new `Actorling` instance that shares network context with the creator.
    /// Useful for creating actors in the same process (possibly/commonly in child threads),
    /// that can talk to the creator actor (usually running on the main thread, but could be
    /// run from a child thread as well).
    pub fn new_sibling(addr: &str, context: zmq::Context) -> Result<Self> {
        let address = addr.to_string();
        let pipe = Socket::new(zmq::PAIR).unwrap();
        let uuid = Uuid::new_v5(&NAMESPACE_DNS, "actorling");
        let _ = pipe.bind(&uuid_pipe_address(&uuid))?;
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
    /// `Actorling::new_sibling`).
    pub fn context(&self) -> zmq::Context {
        self.context.clone()
    }

    /// Returns a `String` the IPC address for this `Actorling`.
    pub fn pipe_address(&self) -> String {
        uuid_pipe_address(&self.uuid)
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

fn uuid_pipe_address(uuid: &Uuid) -> String {
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
    fn actorlings_return_pipe_address_on_start() {
        let acty = Actorling::new("inproc://my_actorling").unwrap();
        let pipe_addr = acty.pipe_address();
        let ep = format!("inproc://{}", acty.uuid.simple().to_string());
        assert_eq!(pipe_addr, ep);
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
