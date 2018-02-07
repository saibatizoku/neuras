//! `tokio`-compatibility for sockets.
use super::{SocketRecv, SocketSend, SocketWrapper};
use super::mio::PollableSocket;

use std::io;
use std::ops::Deref;
use futures::{Async, Future, Poll};
use tokio_core::reactor::{Handle, PollEvented};
use zmq::{Message, Sendable, Socket};

/// A Future that sends a `Message`.
pub struct SendMessage<'a> {
    socket: &'a TokioSocket<'a>,
    message: Message,
    flags: i32,
}

impl<'a> SendMessage<'a> {
    /// Create a new `SendMessage` future.
    pub fn new<M: Into<Message>>(socket: &'a TokioSocket, msg: M, flags: i32) -> SendMessage<'a> {
        let message = msg.into();
        SendMessage { socket, message, flags }
    }
}

impl<'a> Future for SendMessage<'a>
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketSend::send(self.socket, self.message.deref(), self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(_) => Ok(Async::Ready(())),
        }
    }
}

/// A Future that sends a multi-part `Message`.
pub struct SendMultipartMessage<'a> {
    socket: &'a TokioSocket<'a>,
    messages: Vec<Vec<u8>>,
    flags: i32,
}

impl<'a> SendMultipartMessage<'a> {
    /// Create a new `SendMultipartMessage`.
    pub fn new<I, M>(socket: &'a TokioSocket, iter: I, flags: i32) -> SendMultipartMessage<'a>
    where I: IntoIterator<Item = M>,
          M: Into<Vec<u8>>,
    {
        let messages: Vec<Vec<u8>> = iter.into_iter().map(|m| m.into()).collect();
        SendMultipartMessage { socket, messages, flags }
    }
}

impl<'a> Future for SendMultipartMessage<'a> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketSend::send_multipart(self.socket, &self.messages, self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(_) => Ok(Async::Ready(())),
        }
    }
}

/// A Future that receives a `Message` asynchronously. This is returned by `Socket::recv`
pub struct ReceiveMessage<'a> {
    socket: &'a TokioSocket<'a>,
    flags: i32,
}

impl<'a> ReceiveMessage<'a> {
    pub fn new(socket: &'a TokioSocket, flags: i32) -> ReceiveMessage<'a> {
        ReceiveMessage { socket, flags }
    }
}

impl<'a> Future for ReceiveMessage<'a> {
    type Item = Message;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketRecv::recv_msg(self.socket, self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(msg) => Ok(Async::Ready(msg)),
        }
    }
}

/// A Future that receives a multi-part `Message` asynchronously.
/// This is returned by `Socket::recv_multipart`
pub struct ReceiveMultipartMessage<'a> {
    socket: &'a TokioSocket<'a>,
    flags: i32,
}

impl<'a> ReceiveMultipartMessage<'a> {
    pub fn new(socket: &'a TokioSocket, flags: i32) -> ReceiveMultipartMessage<'a> {
        ReceiveMultipartMessage { socket, flags }
    }
}

impl<'a> Future for ReceiveMultipartMessage<'a> {
    type Item = Vec<Message>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketRecv::recv_multipart(self.socket, self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(msgs) => {
                let m_out = msgs.iter().map(|v| v.into()).collect::<Vec<Message>>();
                Ok(Async::Ready(m_out))
            }
        }
    }
}

/// `tokio`-compatible wrapper for sockets.
pub struct TokioSocket<'a> {
    inner: PollEvented<PollableSocket<'a>>,
}

impl<'a> TokioSocket<'a> {
    pub fn new(socket: &'a Socket, handle: &Handle) -> io::Result<TokioSocket<'a>> {
        let inner = PollEvented::new(PollableSocket::new(socket), handle)?;
        Ok(TokioSocket { inner })
    }
}

/// API for socket futures.
pub trait SocketFutures {
    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    fn async_send<M: Into<Message>>(&self, message: M, flags: i32) -> SendMessage;

    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    fn async_send_multipart<I, M>(&self, messages: I, flags: i32) -> SendMultipartMessage
    where
        I: IntoIterator<Item = M>,
        M: Into<Vec<u8>>;

    /// Returns a `Future` that resolves into a `zmq::Message`
    fn async_recv(&self, flags: i32) -> ReceiveMessage;

    /// Returns a `Future` that resolves into a `Vec<zmq::Message>`
    fn async_recv_multipart(&self, flags: i32) -> ReceiveMultipartMessage;
}

impl<'a> SocketFutures for TokioSocket<'a> {
    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    fn async_send<M: Into<Message>>(&self, message: M, flags: i32) -> SendMessage {
        SendMessage::new(self, message, flags)
    }

    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    fn async_send_multipart<I, M>(&self, messages: I, flags: i32) -> SendMultipartMessage
    where
        I: IntoIterator<Item = M>,
        M: Into<Vec<u8>>,
    {
        SendMultipartMessage::new(self, messages, flags)
    }

    /// Returns a `Future` that resolves into a `zmq::Message`
    fn async_recv(&self, flags: i32) -> ReceiveMessage {
        ReceiveMessage::new(self, flags)
    }

    /// Returns a `Future` that resolves into a `Vec<zmq::Message>`
    fn async_recv_multipart(&self, flags: i32) -> ReceiveMultipartMessage {
        ReceiveMultipartMessage::new(self, flags)
    }
}

impl<'a> SocketWrapper for TokioSocket<'a> {
    fn get_socket_ref(&self) -> &Socket {
        SocketWrapper::get_socket_ref(&self.inner)
    }
    fn get_rcvmore(&self) -> io::Result<bool> {
        SocketWrapper::get_rcvmore(&self.inner)
    }
}

impl<'b, T> SocketWrapper for &'b T
where
    T: SocketWrapper + 'b,
{
    fn get_socket_ref(&self) -> &Socket {
        SocketWrapper::get_socket_ref(*self)
    }
    fn get_rcvmore(&self) -> io::Result<bool> {
        SocketWrapper::get_rcvmore(*self)
    }
}

impl<'b, T> SocketSend for &'b T
where
    T: SocketSend + 'b,
{
    fn send<M>(&self, msg: M, flags: i32) -> io::Result<()>
    where
        M: Sendable,
    {
        SocketSend::send(*self, msg, flags)
    }

    fn send_multipart<I, M>(&self, iter: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Into<Message>,
    {
        SocketSend::send_multipart(*self, iter, flags)
    }
}

impl<'a> SocketSend for TokioSocket<'a> {
    fn send<M>(&self, msg: M, flags: i32) -> io::Result<()>
    where
        M: Sendable,
    {
        if let Async::NotReady = self.inner.poll_write() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketSend::send(&self.inner, msg, flags);
        if is_wouldblock(&r) {
            self.inner.need_write();
        }
        return r;
    }

    fn send_multipart<I, M>(&self, iter: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Into<Message>,
    {
        if let Async::NotReady = self.inner.poll_write() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketSend::send_multipart(&self.inner, iter, flags);
        if is_wouldblock(&r) {
            self.inner.need_write();
        }
        return r;
    }
}

impl<'a> SocketRecv for TokioSocket<'a> {
    /// Receive a message into a `Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, buf: &mut Message, flags: i32) -> io::Result<()> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv(&self.inner, buf, flags);
        if is_wouldblock(&r) {
            self.inner.need_read();
        }
        return r;
    }

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_into(&self.inner, buf, flags);
        if is_wouldblock(&r) {
            self.inner.need_read();
        }
        return r;
    }

    /// Receive a message into a fresh `Message`.
    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_msg(&self.inner, flags);
        if is_wouldblock(&r) {
            self.inner.need_read();
        }
        return r;
    }

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_bytes(&self.inner, flags);
        if is_wouldblock(&r) {
            self.inner.need_read();
        }
        return r;
    }

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec` in the `Err`
    /// part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_string(&self.inner, flags);
        if is_wouldblock(&r) {
            self.inner.need_read();
        }
        return r;
    }

    /// Receive a multipart message from the socket.
    ///
    /// Note that this will allocate a new vector for each message part; for many applications it
    /// will be possible to process the different parts sequentially and reuse allocations that
    /// way.
    fn recv_multipart(&self, flags: i32) -> io::Result<Vec<Vec<u8>>> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_multipart(&self.inner, flags);
        if is_wouldblock(&r) {
            self.inner.need_read();
        }
        return r;
    }
}

impl<'a, 'b> SocketRecv for &'b TokioSocket<'a> {
    /// Receive a message into a `Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, buf: &mut Message, flags: i32) -> io::Result<()> {
        SocketRecv::recv(*self, buf, flags)
    }

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        SocketRecv::recv_into(*self, buf, flags)
    }

    /// Receive a message into a fresh `Message`.
    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        SocketRecv::recv_msg(*self, flags)
    }

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        SocketRecv::recv_bytes(*self, flags)
    }

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec` in the `Err`
    /// part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        SocketRecv::recv_string(*self, flags)
    }

    /// Receive a multipart message from the socket.
    ///
    /// Note that this will allocate a new vector for each message part; for many applications it
    /// will be possible to process the different parts sequentially and reuse allocations that
    /// way.
    fn recv_multipart(&self, flags: i32) -> io::Result<Vec<Vec<u8>>> {
        SocketRecv::recv_multipart(*self, flags)
    }
}

impl<'a, T: SocketWrapper + 'a> SocketWrapper for PollEvented<T> {
    fn get_socket_ref(&self) -> &Socket {
        SocketWrapper::get_socket_ref(self.get_ref())
    }
    fn get_rcvmore(&self) -> io::Result<bool> {
        SocketWrapper::get_rcvmore(self.get_ref())
    }
}

impl<'a, T: SocketSend + 'a> SocketSend for PollEvented<T> {
    fn send<M>(&self, msg: M, flags: i32) -> io::Result<()>
    where
        M: Sendable,
    {
        if let Async::NotReady = self.poll_write() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketSend::send(self.get_ref(), msg, flags);
        if is_wouldblock(&r) {
            self.need_write();
        }
        return r;
    }

    fn send_multipart<I, M>(&self, iter: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Into<Message>,
    {
        if let Async::NotReady = self.poll_write() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketSend::send_multipart(self.get_ref(), iter, flags);
        if is_wouldblock(&r) {
            self.need_write();
        }
        return r;
    }
}

impl<'a, T: SocketRecv + 'a> SocketRecv for PollEvented<T> {
    /// Receive a message into a `Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, buf: &mut Message, flags: i32) -> io::Result<()> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv(self.get_ref(), buf, flags);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_into(self.get_ref(), buf, flags);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }

    /// Receive a message into a fresh `Message`.
    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_msg(self.get_ref(), flags);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_bytes(self.get_ref(), flags);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec` in the `Err`
    /// part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_string(self.get_ref(), flags);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }

    /// Receive a multipart message from the socket.
    ///
    /// Note that this will allocate a new vector for each message part; for many applications it
    /// will be possible to process the different parts sequentially and reuse allocations that
    /// way.
    fn recv_multipart(&self, flags: i32) -> io::Result<Vec<Vec<u8>>> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let r = SocketRecv::recv_multipart(self.get_ref(), flags);
        if is_wouldblock(&r) {
            self.need_read();
        }
        return r;
    }
}

impl<'a, 'b> From<(&'a Socket, &'b Handle)> for TokioSocket<'a> {
    fn from(socket_n_handle: (&'a Socket, &'b Handle)) -> Self {
        let (socket, handle) = socket_n_handle;
        TokioSocket::new(socket, handle).unwrap()
    }
}

// Convenience function to check if messaging will block or not.
fn is_wouldblock<T>(r: &io::Result<T>) -> bool {
    match *r {
        Ok(_) => false,
        Err(ref e) => e.kind() == io::ErrorKind::WouldBlock,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_core::reactor::Core;
    use zmq::{self, Context, Socket};

    fn setup_socket() -> (Socket, Core) {
        let ctx = Context::new();
        let core = Core::new().unwrap();
        let socket = ctx.socket(zmq::PAIR).unwrap();
        let _ = socket.set_identity(b"my_identity").unwrap();
        (socket, core)
    }

    #[test]
    fn new_tokio_socket_wraps_reference_to_zmq_socket() {
        let (socket, core) = setup_socket();
        let handle = core.handle();
        let tokio = TokioSocket::new(&socket, &handle).unwrap();
        assert_eq!(tokio.get_socket_ref().get_identity(), socket.get_identity());
    }

    #[test]
    fn convert_from_zmq_socket_reference_to_tokio_socket() {
        let (socket, core) = setup_socket();
        let handle = core.handle();
        let tokio: TokioSocket = (&socket, &handle).into();
        assert_eq!(tokio.get_socket_ref().get_identity(), socket.get_identity());
    }
}
