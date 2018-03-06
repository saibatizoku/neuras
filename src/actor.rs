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
//!


use std::collections::VecDeque;
use std::thread;
use std::time;
use uuid::Uuid;
use zmq;

/// A mailbox where every incoming message goes through.
#[derive(Debug, Default, PartialEq)]
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
        let uuid = Uuid::new_v4();
        let actorling = Actorling {
            address,
            context,
            pipe,
            uuid,
        };
        Ok(actorling)
    }
}

impl Default for Actorling {
    fn default() -> Self {
        Self::new("").unwrap()
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

            run_blocking_poll(&pipe, &service, &mut mbox, 10)
        })?;
        Ok(handle)
    }

    /// Stops the current actorling instance.
    pub fn stop(&self) -> Result<()> {
        self.pipe()
            .send("$STOP", 0)
            .chain_err(|| "could not send stop command")
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

pub fn run_blocking_poll(
    pipe: &zmq::Socket,
    service: &zmq::Socket,
    mbox: &mut Mailbox,
    timeout: i64,
) -> Result<()> {
    let mut pollable = [
        pipe.as_poll_item(zmq::POLLIN),
        service.as_poll_item(zmq::POLLIN),
    ];

    let mut msg = zmq::Message::new();

    loop {
        zmq::poll(&mut pollable, timeout)?;
        if pollable[0].is_readable() {
            pipe.recv(&mut msg, 0)?;
            match &*msg {
                b"PING" => {
                    pipe.send("PONG", 0)?;
                }
                b"$STOP" => {
                    pipe.send("OK", 0)?;
                    break;
                }
                _ => {
                    pipe.send("ERR", 0)?;
                }
            }
        }
        if pollable[1].is_readable() {
            let msg = service.recv_multipart(0).unwrap();
            mbox.push_back(msg);
        }
        // read a message from the inbox
        if let Some(next_msg) = mbox.pop_front() {
            println!("reading {:?}", next_msg);
        }
    }
    Ok(())
}

fn parse_pipe_command(pipe: &zmq::Socket, msg: &[u8]) -> Result<()> {
    match msg {
        b"PING" => pipe.send("PONG", 0)?,
        b"$STOP" => {
            pipe.send("OK", 0)?;
            bail!("interrupted!");
        }
        _ => pipe.send("ERR", 0)?,
    }
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
