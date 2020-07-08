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

use super::socket::{PollingSocket, SocketRecv, SocketWrapper};
use super::utils::run_named_thread;

use failure::Error;
use std::collections::VecDeque;
use std::io;
use std::thread;
use uuid::Uuid;
use zmq;

const PIPE_ADDR: &str = "inproc://neuras.actor.pipe";

/// Actorling Errors.
#[derive(Debug, Fail)]
pub enum ActorlingError {
    #[fail(display = "actorling was interrupted")]
    Interrupted,
    #[fail(display = "invalid command")]
    InvalidCommand,
    #[fail(display = "{}", _0)]
    SocketSend(#[cause] zmq::Error),
}

/// A mailbox where every incoming message goes through.
#[derive(Debug, Default, PartialEq)]
pub struct Mailbox {
    inbox: VecDeque<Vec<Vec<u8>>>,
    outbox: VecDeque<PipeCommand>,
}

impl Mailbox {}

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
    pub fn new(addr: &str) -> Result<Self, Error> {
        Actorling::new_with_context(addr, zmq::Context::new())
    }

    /// Create a new `Actorling` instance that shares network context with the creator.
    /// Useful for creating actors in the same process (possibly/commonly in child threads),
    /// that can talk to the creator actor (usually running on the main thread, but could be
    /// run from a child thread as well).
    pub fn new_with_context(addr: &str, context: zmq::Context) -> Result<Self, Error> {
        let address = addr.to_string();
        let pipe = context.socket(zmq::PAIR)?;
        pipe.connect(PIPE_ADDR)?;
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

    /// Return a reference to the underlying pipe socket.
    pub fn pipe(&self) -> &zmq::Socket {
        &self.pipe
    }

    /// Return a reference to the underlying pipe socket.
    pub fn pollable_pipe(self) -> PollingSocket {
        PollingSocket::new(self.pipe)
    }

    /// Start the current actorling instance.
    pub fn start(&self) -> Result<thread::JoinHandle<Result<(), Error>>, io::Error> {
        // We create a new UUID that will only be known to each PAIR socket at runtime.
        let context = self.context();
        let address = self.address();
        let mut mbox = Mailbox::default();

        run_named_thread("pipe", move || {
            let pipe = context.socket(zmq::PAIR)?;
            pipe.bind(PIPE_ADDR)?;

            let service = context.socket(zmq::PULL)?;
            service.bind(&address)?;
            let pub_addr = service
                .get_last_endpoint()?
                .expect("unparsable actor endpoint");
            pipe.send(&pub_addr, 0)?;

            poll_zmq_actor(pipe, service, &mut mbox, 10)
        })
    }

    /// Stop the current actorling instance.
    pub fn stop(&self) -> Result<(), zmq::Error> {
        self.pipe().send("$STOP", 0)
    }

    pub fn pop(&self) -> Result<Option<Vec<zmq::Message>>, Error> {
        self.pipe().send("$POP", 0)?;
        let mut msgs = Vec::<zmq::Message>::new();
        let msg = self.pipe().recv_msg(0)?;
        match &*msg {
            b"$NONE" => Ok(None),
            _ => {
                msgs.push(msg);
                while self.pipe().get_rcvmore()? {
                    let msg = self.pipe().recv_msg(0)?;
                    msgs.push(msg)
                }
                Ok(Some(msgs))
            }
        }
    }

    /// Returns the actorling's UUID as a `String`
    pub fn uuid(&self) -> String {
        self.uuid.to_simple().to_string()
    }
}

pub fn poll_zmq_actor(
    pipe: zmq::Socket,
    service: zmq::Socket,
    mbox: &mut Mailbox,
    timeout: i64,
) -> Result<(), Error> {
    let p = PollingSocket::new(pipe);
    let s = PollingSocket::new(service);
    let mut pollable = [
        p.get_socket_ref().as_poll_item(zmq::POLLIN),
        s.get_socket_ref().as_poll_item(zmq::POLLIN),
    ];

    let mut msg = zmq::Message::new();

    loop {
        zmq::poll(&mut pollable, timeout)?;
        if pollable[0].is_readable() {
            if let Err(e) = p.recv(&mut msg, 0) {
                match e.kind() {
                    io::ErrorKind::WouldBlock => continue,
                    _ => bail!("actor pipe could not be read"),
                }
            };

            let cmd = parse_pipe_command(&*msg)?;
            println!("command: {:?}", cmd);

            if let Err(e) = execute_command(p.get_socket_ref(), &cmd) {
                match e {
                    ActorlingError::Interrupted => break,
                    ActorlingError::InvalidCommand => continue,
                    _ => bail!(e),
                }
            };
        }
        if pollable[1].is_readable() {
            loop {
                match s.recv_multipart(0) {
                    Ok(msg) => mbox.inbox.push_back(msg),
                    Err(e) => match e.kind() {
                        io::ErrorKind::WouldBlock => break,
                        _ => bail!("actor service could not be read"),
                    },
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum PipeCommand {
    Interrupt,
    Invalid,
    Send(&'static str),
}

fn parse_pipe_command(msg: &[u8]) -> Result<PipeCommand, Error> {
    let cmd = match msg {
        b"$PING" => PipeCommand::Send("$PONG"),
        b"$STOP" => PipeCommand::Interrupt,
        _ => PipeCommand::Invalid,
    };
    Ok(cmd)
}

fn execute_command(pipe: &zmq::Socket, cmd: &PipeCommand) -> Result<(), ActorlingError> {
    match *cmd {
        PipeCommand::Send(message) => pipe.send(message, 0).map_err(ActorlingError::SocketSend)?,
        PipeCommand::Interrupt => {
            pipe.send("$STOPPING", 0)
                .map_err(ActorlingError::SocketSend)?;
            return Err(ActorlingError::Interrupted);
        }
        PipeCommand::Invalid => {
            pipe.send("$WONTDO", 0)
                .map_err(ActorlingError::SocketSend)?;
            return Err(ActorlingError::InvalidCommand);
        }
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
