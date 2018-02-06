//! `mio`-compatibility for sockets.
use super::{SocketRecv, SocketSend, SocketWrapper};

use std::io;
use std::os::unix::io::RawFd;

use mio_lib::unix::EventedFd;
use mio_lib::{Evented, Poll, PollOpt, Ready, Token};
use zmq::{Message, Sendable, Socket, DONTWAIT};

/// `mio`-compatible wrapper for sockets.
pub struct PollableSocket<'a> {
    inner: &'a Socket,
}

impl<'a> PollableSocket<'a> {
    pub fn new(inner: &'a Socket) -> PollableSocket {
        PollableSocket { inner }
    }

    pub fn as_fd(&self) -> io::Result<RawFd> {
        let fd = self.inner.get_fd()?;
        Ok(fd)
    }

    pub fn get_ref(&self) -> &Socket {
        self.inner
    }
}

impl<'a> SocketWrapper for PollableSocket<'a> {
    fn get_socket_ref(&self) -> &Socket {
        self.get_ref()
    }
}

impl<'a> SocketSend for PollableSocket<'a> {
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

impl<'a> SocketRecv for PollableSocket<'a> {
    /// Return true if there are more frames of a multipart message to receive.
    fn get_rcvmore(&self) -> io::Result<bool> {
        self.get_socket_ref().get_rcvmore().map_err(|e| e.into())
    }

    /// Receive a message into a `Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, buf: &mut Message, flags: i32) -> io::Result<()> {
        self.get_socket_ref()
            .recv(buf, DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        self.get_socket_ref()
            .recv_into(buf, DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    /// Receive a message into a fresh `Message`.
    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        self.get_socket_ref()
            .recv_msg(DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        self.get_socket_ref()
            .recv_bytes(DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec` in the `Err`
    /// part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        self.get_socket_ref()
            .recv_string(DONTWAIT | flags)
            .map_err(|e| e.into())
    }

    /// Receive a multipart message from the socket.
    ///
    /// Note that this will allocate a new vector for each message part; for many applications it
    /// will be possible to process the different parts sequentially and reuse allocations that
    /// way.
    fn recv_multipart(&self, flags: i32) -> io::Result<Vec<Vec<u8>>> {
        self.get_socket_ref()
            .recv_multipart(DONTWAIT | flags)
            .map_err(|e| e.into())
    }
}

impl<'a> Evented for PollableSocket<'a> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let fd = try!(self.as_fd());
        EventedFd(&fd).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let fd = try!(self.as_fd());
        EventedFd(&fd).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        let fd = try!(self.as_fd());
        EventedFd(&fd).deregister(poll)
    }
}

impl<'a, 'b> Evented for &'b PollableSocket<'a> {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let fd = (*self).as_fd()?;
        EventedFd(&fd).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let fd = try!((*self).as_fd());
        EventedFd(&fd).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        let fd = try!((*self).as_fd());
        EventedFd(&fd).deregister(poll)
    }
}

impl<'a> From<&'a Socket> for PollableSocket<'a> {
    fn from(socket: &'a Socket) -> Self {
        PollableSocket::new(socket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zmq::{self, Context, Socket};

    fn setup_socket() -> Socket {
        let ctx = Context::new();
        let socket = ctx.socket(zmq::PAIR).unwrap();
        let _ = socket.set_identity(b"my_identity").unwrap();
        socket
    }

    #[test]
    fn new_mio_socket_wraps_reference_to_zmq_socket() {
        let socket = setup_socket();
        let evented = PollableSocket::new(&socket);
        assert_eq!(evented.inner.get_identity(), socket.get_identity());
    }

    #[test]
    fn convert_from_zmq_socket_reference_to_mio_socket() {
        let socket = setup_socket();
        let evented: PollableSocket = (&socket).into();
        assert_eq!(evented.inner.get_identity(), socket.get_identity());
    }
}
