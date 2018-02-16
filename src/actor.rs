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
//!     // Initialize a re-usable message and our actor,
//!     // which will be listening on "tcp://127.0.0.1:11227".
//!     let mut msg = zmq::Message::new();
//!     let actorling = Actorling::new("tcp://127.0.0.1:11227").unwrap();
//!
//!     // Starting the actor returns the `std::thread::JoinHandle<_>`
//!     // for the thread where all the actual I/O is happening.
//!     let thread_handle = actorling.start().unwrap();
//!
//!     // Get a reference to the pipe connecting to the child-thread.
//!     let pipe = actorling.pipe();
//!
//!     // Now we can go crazy by sending messages from the actor into
//!     // the child-thread...
//!
//!     // For example: we send `PING`, and expect a `PONG` in return.
//!     {
//!         pipe.send("PING", 0).unwrap();
//!         pipe.recv(&mut msg, 0).unwrap();
//!         let status = msg.as_str().unwrap();
//!
//!         println!("status: {}", &status);
//!         assert_eq!("PONG", status);
//!     }
//!
//!     // or, we send `$STOP` and expect `OK`, meaning the child-thread
//!     // will exit cleanly. Which means we can join it into this main thread,
//!     // and tidy up after ourselves.
//!     {
//!         pipe.send("$STOP", 0).unwrap();
//!         pipe.recv(&mut msg, 0).unwrap();
//!         let status = msg.as_str().unwrap();
//!
//!         println!("status: {}", &status);
//!         assert_eq!("OK", status);
//!     }
//!
//!     // The child-thread joins `Ok`.
//!     assert!(thread_handle.join().is_ok());
//!
//!     // trying to stop a stopped actorling will return ok, but print
//!     // a message to stderr.
//!     assert!(actorling.stop().is_ok());
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

use std::collections::VecDeque;
use std::thread;
use std::time;
use uuid::{Uuid, NAMESPACE_DNS};
use zmq;

#[derive(Default)]
pub struct Mailbox {
    messages: VecDeque<Vec<Vec<u8>>>,
}

impl Mailbox {
    fn push_back(&mut self, msg: Vec<Vec<u8>>) {
        self.messages.push_back(msg)
    }

    fn pop_front(&mut self) -> Option<Vec<Vec<u8>>> {
        self.messages.pop_front()
    }
}

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
        let pipe = context.socket(zmq::PAIR)?;
        pipe.connect("inproc://neuras.actor.pipe")?;
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

    pub fn pipe(&self) -> &zmq::Socket {
        &self.pipe
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
    pub fn start(&self) -> Result<thread::JoinHandle<Result<()>>> {
        // We create a new UUID that will only be known to each PAIR socket at runtime.
        let context = self.context();
        let address = self.address();
        let mut mbox = Mailbox::default();

        let handle = run_thread("pipe", move || {
            let pipe = context.socket(zmq::PAIR).unwrap();
            pipe.bind("inproc://neuras.actor.pipe").unwrap();
            let service = context.socket(zmq::PULL).unwrap();
            service.bind(&address).unwrap();

            let mut pollable = [
                pipe.as_poll_item(zmq::POLLIN),
                service.as_poll_item(zmq::POLLIN),
            ];

            let mut msg = zmq::Message::new();

            loop {
                zmq::poll(&mut pollable, 10).unwrap();
                if pollable[0].is_readable() {
                    pipe.recv(&mut msg, 0).unwrap();
                    match &*msg {
                        b"PING" => {
                            pipe.send("PONG", 0).unwrap();
                        }
                        b"$STOP" => {
                            pipe.send("OK", 0).unwrap();
                            break;
                        }
                        _ => {
                            pipe.send("ERR", 0).unwrap();
                        }
                    }
                }
                if pollable[1].is_readable() {
                    let msg = service.recv_multipart(0).unwrap();
                    mbox.push_back(msg);
                }
            }
            Ok(())
        })?;
        Ok(handle)
    }

    /// Stops the current actorling instance.
    pub fn stop(&self) -> Result<()> {
        let pipe = self.pipe();

        let max_wait = time::Duration::from_millis(200); // 200 ms timeout
        let started = time::Instant::now();
        loop {
            match pipe.poll(zmq::POLLOUT, 10) {
                Ok(_) => {
                    match pipe.send("$STOP", zmq::DONTWAIT) {
                        Err(ref e) if e == &zmq::Error::EAGAIN => {
                            println!("socket would block when sending");
                            let now = time::Instant::now();
                            if now.duration_since(started) < max_wait {
                                continue;
                            } else {
                                bail!(
                                    "actor could not send stop command. it may be already stopped"
                                );
                            }
                        }
                        Ok(_) => {
                            println!("STOP sent");
                            let started = time::Instant::now();
                            loop {
                                match pipe.poll(zmq::POLLIN, 10) {
                                    Ok(_) => {
                                        match pipe.recv_msg(zmq::DONTWAIT) {
                                            Err(ref e) if e == &zmq::Error::EAGAIN => {
                                                //println!("socket would block receiving");
                                                let now = time::Instant::now();
                                                if now.duration_since(started) < max_wait {
                                                    continue;
                                                }
                                                eprintln!("actor could not confirm stop command. it may be already stopped");
                                                return Ok(());
                                            }
                                            Err(e) => {
                                                println!(
                                                    "error receiving stop confirmation: {:?}",
                                                    e
                                                );
                                                bail!(e);
                                            }
                                            _ => println!("stop confirmed"),
                                        }
                                    }
                                    _ => bail!("pipe POLLIN error"),
                                }
                            }
                        }
                        Err(e) => {
                            println!("stopping error: {:?}", e);
                            bail!(e);
                        }
                    }
                }
                _ => bail!("pipe POLLOUT error"),
            }
        }
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
    thread::Builder::new()
        .name(name.to_string())
        .spawn(callback)
        .chain_err(|| "could not spawn actorling thread")
}

/// Try sending a `zmq::Sendable` message before the timeout expires.
pub fn polled_send<M: zmq::Sendable>(
    socket: &zmq::Socket,
    msg: M,
    flags: i32,
    timeout: i32,
) -> Result<()> {
    Ok(())
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
    }

    #[test]
    fn actorlings_join_thread_on_stop() {
        let acty = Actorling::new("inproc://my_actorling").unwrap();
        let handle = acty.start().unwrap();
        acty.stop().unwrap();
        assert!(handle.join().is_ok());
    }

    #[test]
    fn actorlings_return_ok_if_stopped_when_not_running() {
        let acty = Actorling::new("inproc://my_actorling").unwrap();
        let stop = acty.stop();
        assert!(stop.is_ok());
    }
}
