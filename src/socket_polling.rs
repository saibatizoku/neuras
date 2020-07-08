//! Non-blocking sockets for polling.
//!
//! `PollingSocket` implements `SocketSend`, and `SocketRecv` by sending
//! the `zmq::DONTWAIT` flag, and returning `std::io::Error`.
//!
//! Users of `PollingSocket` must handle the case when the result is an error
//! with `std::io::ErrorKind::WouldBlock`, as per-usual when dealing with
//! non-blocking (asynchronous) code.
//!
//! This module also adds `mio`-compatibility for sockets, by implementing
//! the `mio::Evented` trait, which is used for registering the
//! socket with a `mio::Poll` instance.
use super::{SocketRecv, SocketSend, SocketWrapper};

use std::io;
use std::os::unix::io::RawFd;

use mio_lib::unix::EventedFd;
use mio_lib::Evented;
use mio_lib::{Poll, PollOpt, Ready, Token};
use zmq::{Message, Sendable, Socket, DONTWAIT};

/// Socket used for polling with `mio::Poll`.
pub struct PollingSocket {
    inner: Socket,
}

impl PollingSocket {
    /// Create a new `PollingSocket` instance.
    pub fn new(inner: Socket) -> PollingSocket {
        PollingSocket { inner }
    }

    /// Return a result with the `RawFd` from the underlying socket.
    pub fn as_fd(&self) -> io::Result<RawFd> {
        let fd = self.inner.get_fd()?;
        Ok(fd)
    }
}

/// Implementation of the `SocketWrapper` API for pollable sockets.
impl SocketWrapper for PollingSocket {
    fn get_socket_ref(&self) -> &Socket {
        &self.inner
    }
    fn get_rcvmore(&self) -> io::Result<bool> {
        self.get_socket_ref().get_rcvmore().map_err(|e| e.into())
    }
}

/// Implementation of the `SocketSend` API for pollable sockets.
impl SocketSend for PollingSocket {
    fn send<M>(&self, msg: M, flags: i32) -> io::Result<()>
    where
        M: Sendable,
    {
        self.get_socket_ref()
            .send(msg, DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    fn send_multipart<I, M>(&self, iter: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Into<Message>,
    {
        self.get_socket_ref()
            .send_multipart(iter, DONTWAIT | flags)
            .map_err(|e| e.into())
    }
}

/// Implementation of the `SocketRecv` API for pollable sockets.
impl SocketRecv for PollingSocket {
    fn recv(&self, buf: &mut Message, flags: i32) -> io::Result<()> {
        self.get_socket_ref()
            .recv(buf, DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        self.get_socket_ref()
            .recv_into(buf, DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        self.get_socket_ref()
            .recv_msg(DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        self.get_socket_ref()
            .recv_bytes(DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        self.get_socket_ref()
            .recv_string(DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    fn recv_multipart(&self, flags: i32) -> io::Result<Vec<Vec<u8>>> {
        self.get_socket_ref()
            .recv_multipart(DONTWAIT | flags)
            .map_err(|e| e.into())
    }
}

/// Converts from a regular socket into a pollable socket.
impl From<Socket> for PollingSocket {
    fn from(socket: Socket) -> Self {
        PollingSocket::new(socket)
    }
}

/// Converts from a pollable socket into a regular socket.
impl Into<Socket> for PollingSocket {
    fn into(self) -> Socket {
        self.inner
    }
}

/// Implementation of the external `mio::Evented` API for pollable sockets.
impl Evented for PollingSocket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let fd = self.as_fd()?;
        EventedFd(&fd).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let fd = self.as_fd()?;
        EventedFd(&fd).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        let fd = self.as_fd()?;
        EventedFd(&fd).deregister(poll)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zmq::{self, Context, Socket};

    fn setup_socket() -> Socket {
        let ctx = Context::new();
        let socket = ctx.socket(zmq::PAIR).unwrap();
        socket.set_identity(b"my_identity").unwrap();
        socket
    }

    #[test]
    fn new_pollable_socket_wraps_reference_to_zmq_socket() {
        let socket = setup_socket();
        let pollable = PollingSocket::new(socket);
        assert_eq!(pollable.inner.get_identity(), Ok(b"my_identity".to_vec()));
    }

    #[test]
    fn convert_from_zmq_socket_reference_to_pollable_socket() {
        let socket = setup_socket();
        let pollable: PollingSocket = socket.into();
        assert_eq!(pollable.inner.get_identity(), Ok(b"my_identity".to_vec()));
    }
}
