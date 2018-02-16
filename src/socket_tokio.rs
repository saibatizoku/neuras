//! `tokio`-compatibility for sockets.
#[path = "socket_tokio_future.rs"]
pub mod future;
#[path = "socket_tokio_stream.rs"]
pub mod stream;
#[path = "socket_tokio_sink.rs"]
pub mod sink;

use self::future::{SendMessage, SendMultipartMessage};
use self::future::{RecvMessage, RecvMultipartMessage};
use self::stream::{MessageMultipartStream, MessageStream};
use self::sink::{MessageMultipartSink, MessageSink};
use super::{SocketRecv, SocketSend, SocketWrapper};
use super::mio::PollableSocket;

use std::io;
use futures::Async;
use tokio_core::reactor::{Handle, PollEvented};
use zmq::{Message, Sendable, Socket};

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

impl<'a> TokioSocket<'a> {
    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    pub fn send<M: Into<Message>>(&self, message: M, flags: i32) -> SendMessage {
        SendMessage::new(self, message, flags)
    }

    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    pub fn send_multipart<I, M>(&self, messages: I, flags: i32) -> SendMultipartMessage
    where
        I: IntoIterator<Item = M>,
        M: Into<Vec<u8>>,
    {
        SendMultipartMessage::new(self, messages, flags)
    }

    /// Returns a `Future` that resolves into a `zmq::Message`
    pub fn recv<'b>(&'a self, msg: &'b mut Message, flags: i32) -> RecvMessage<'a, 'b> {
        RecvMessage::new(self, msg, flags)
    }

    /// Returns a `Future` that resolves into a `Vec<zmq::Message>`
    pub fn recv_multipart(&self, flags: i32) -> RecvMultipartMessage {
        RecvMultipartMessage::new(self, flags)
    }

    /// Returns a `Stream` of incoming messages.
    pub fn stream(&self) -> MessageStream<Self> {
        MessageStream::new(self)
    }

    /// Returns a `Stream` of incoming multi-part messages.
    pub fn stream_multipart(&self) -> MessageMultipartStream<Self> {
        MessageMultipartStream::new(self)
    }

    /// Returns a `Sink` for outgoing messages.
    pub fn sink(&self) -> MessageSink<Self> {
        MessageSink::new(self)
    }

    /// Returns a `Sink` for outgoing multi-part messages.
    pub fn sink_multipart(&self) -> MessageMultipartSink<Self> {
        MessageMultipartSink::new(self)
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
        let resulting = SocketSend::send(&self.inner, msg, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_write();
        }
        resulting
    }

    fn send_multipart<I, M>(&self, iter: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Into<Message>,
    {
        if let Async::NotReady = self.inner.poll_write() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketSend::send_multipart(&self.inner, iter, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_write();
        }
        resulting
    }
}

impl<'a> SocketRecv for TokioSocket<'a> {
    /// Receive a message into a `Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, buf: &mut Message, flags: i32) -> io::Result<()> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv(&self.inner, buf, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_read();
        }
        resulting
    }

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_into(&self.inner, buf, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_read();
        }
        resulting
    }

    /// Receive a message into a fresh `Message`.
    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_msg(&self.inner, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_read();
        }
        resulting
    }

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_bytes(&self.inner, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_read();
        }
        resulting
    }

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec`
    /// in the `Err` part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        if let Async::NotReady = self.inner.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_string(&self.inner, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_read();
        }
        resulting
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
        let resulting = SocketRecv::recv_multipart(&self.inner, flags);
        if is_wouldblock(&resulting) {
            self.inner.need_read();
        }
        resulting
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
    /// If the received message is not valid UTF-8, it is returned as the original `Vec`
    /// in the `Err` part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        SocketRecv::recv_string(*self, flags)
    }

    /// Receive a multipart message from the socket.
    ///
    /// Note that this will allocate a new vector for each message part; for many
    /// applications it will be possible to process the different parts sequentially
    /// and reuse allocations that way.
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
        let resulting = SocketSend::send(self.get_ref(), msg, flags);
        if is_wouldblock(&resulting) {
            self.need_write();
        }
        resulting
    }

    fn send_multipart<I, M>(&self, iter: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = M>,
        M: Into<Message>,
    {
        if let Async::NotReady = self.poll_write() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketSend::send_multipart(self.get_ref(), iter, flags);
        if is_wouldblock(&resulting) {
            self.need_write();
        }
        resulting
    }
}

impl<'a, T: SocketRecv + 'a> SocketRecv for PollEvented<T> {
    /// Receive a message into a `Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, buf: &mut Message, flags: i32) -> io::Result<()> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv(self.get_ref(), buf, flags);
        if is_wouldblock(&resulting) {
            self.need_read();
        }
        resulting
    }

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_into(self.get_ref(), buf, flags);
        if is_wouldblock(&resulting) {
            self.need_read();
        }
        resulting
    }

    /// Receive a message into a fresh `Message`.
    fn recv_msg(&self, flags: i32) -> io::Result<Message> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_msg(self.get_ref(), flags);
        if is_wouldblock(&resulting) {
            self.need_read();
        }
        resulting
    }

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_bytes(self.get_ref(), flags);
        if is_wouldblock(&resulting) {
            self.need_read();
        }
        resulting
    }

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec`
    /// in the `Err` part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<Result<String, Vec<u8>>> {
        if let Async::NotReady = self.poll_read() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let resulting = SocketRecv::recv_string(self.get_ref(), flags);
        if is_wouldblock(&resulting) {
            self.need_read();
        }
        resulting
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
        let resulting = SocketRecv::recv_multipart(self.get_ref(), flags);
        if is_wouldblock(&resulting) {
            self.need_read();
        }
        resulting
    }
}

impl<'a, 'b> From<(&'a Socket, &'b Handle)> for TokioSocket<'a> {
    fn from(socket_n_handle: (&'a Socket, &'b Handle)) -> Self {
        let (socket, handle) = socket_n_handle;
        TokioSocket::new(socket, handle).unwrap()
    }
}

// Convenience function to check if messaging will block or not.
fn is_wouldblock<T>(resulting: &io::Result<T>) -> bool {
    match *resulting {
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
        socket.set_identity(b"my_identity").unwrap();
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
