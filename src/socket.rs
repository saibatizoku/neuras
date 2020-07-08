//! Convenient ZMQ sockets.
//!
//! A high-level socket API that hides regular `zmq::Context` and `zmq::Socket`.
//!
//! Inspired by [zsock](http://czmq.zeromq.org/czmq4-0:zsock).
use std::io;
use std::result;
use zmq;

#[path = "socket_polling.rs"]
mod polling;

pub use self::polling::PollingSocket;

#[cfg(feature = "async-tokio")]
#[path = "socket_tokio.rs"]
pub mod tokio;

/// Socket Errors.
#[derive(Debug, Fail)]
pub enum SocketError {
    #[fail(display = "{:?}", _0)]
    Endpoint(Vec<u8>),
    #[fail(display = "{}", _0)]
    Zmq(#[cause] zmq::Error),
}

impl From<zmq::Error> for SocketError {
    fn from(e: zmq::Error) -> SocketError {
        SocketError::Zmq(e)
    }
}

/// API for socket-wrapper types.
pub trait SocketWrapper {
    /// Send a message.
    ///
    /// Due to the provided From implementations, this works for `&[u8]`, `Vec<u8>` and `&str`,
    /// as well as on `zmq::Message` itself.
    fn get_socket_ref(&self) -> &zmq::Socket;

    /// Return true if there are more frames of a multipart message to receive.
    fn get_rcvmore(&self) -> io::Result<bool>;
}

/// API methods for sending messages with sockets.
pub trait SocketSend: SocketWrapper {
    /// Send a message.
    ///
    /// Due to the provided From implementations, this works for `&[u8]`, `Vec<u8>` and `&str`,
    /// as well as on `zmq::Message` itself.
    fn send<T>(&self, T, i32) -> io::Result<()>
    where
        T: zmq::Sendable;
    /// Sends a multipart-message.
    fn send_multipart<I, T>(&self, I, i32) -> io::Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Into<zmq::Message>;
}

/// API methods for receiving messages with sockets.
pub trait SocketRecv: SocketWrapper {
    /// Receive a message into a `zmq::Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, &mut zmq::Message, i32) -> io::Result<()>;

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, &mut [u8], i32) -> io::Result<usize>;

    /// Receive a message into a fresh `zmq::Message`.
    fn recv_msg(&self, i32) -> io::Result<zmq::Message>;

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, i32) -> io::Result<Vec<u8>>;

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec` in the `Err`
    /// part of the inner result.
    fn recv_string(&self, i32) -> io::Result<result::Result<String, Vec<u8>>>;

    /// Receive a multipart message from the socket.
    ///
    /// Note that this will allocate a new vector for each message part; for many applications it
    /// will be possible to process the different parts sequentially and reuse allocations that
    /// way.
    fn recv_multipart(&self, i32) -> io::Result<Vec<Vec<u8>>>;
}

/// API declaration for the standard socket.
impl SocketWrapper for zmq::Socket {
    fn get_socket_ref(&self) -> &zmq::Socket {
        self
    }

    fn get_rcvmore(&self) -> io::Result<bool> {
        self.get_rcvmore().map_err(|e| e.into())
    }
}

impl SocketSend for zmq::Socket {
    /// Send a message.
    ///
    /// Due to the provided From implementations, this works for `&[u8]`, `Vec<u8>` and `&str`,
    /// as well as on `zmq::Message` itself.
    fn send<T>(&self, msg: T, flags: i32) -> io::Result<()>
    where
        T: zmq::Sendable,
    {
        self.send(msg, flags).map_err(|e| e.into())
    }

    /// Sends a multipart-message.
    fn send_multipart<I, T>(&self, msg: I, flags: i32) -> io::Result<()>
    where
        I: IntoIterator<Item = T>,
        T: Into<zmq::Message>,
    {
        self.send_multipart(msg, flags).map_err(|e| e.into())
    }
}

/// API methods for receiving messages with sockets.
impl SocketRecv for zmq::Socket {
    /// Receive a message into a `zmq::Message`. The length passed to `zmq_msg_recv` is the length
    /// of the buffer.
    fn recv(&self, msg: &mut zmq::Message, flags: i32) -> io::Result<()> {
        self.recv(msg, flags).map_err(|e| e.into())
    }

    /// Receive bytes into a slice. The length passed to `zmq_recv` is the length of the slice. The
    /// return value is the number of bytes in the message, which may be larger than the length of
    /// the slice, indicating truncation.
    fn recv_into(&self, msg: &mut [u8], flags: i32) -> io::Result<usize> {
        self.recv_into(msg, flags).map_err(|e| e.into())
    }

    /// Receive a message into a fresh `zmq::Message`.
    fn recv_msg(&self, flags: i32) -> io::Result<zmq::Message> {
        self.recv_msg(flags).map_err(|e| e.into())
    }

    /// Receive a message as a byte vector.
    fn recv_bytes(&self, flags: i32) -> io::Result<Vec<u8>> {
        self.recv_bytes(flags).map_err(|e| e.into())
    }

    /// Receive a `String` from the socket.
    ///
    /// If the received message is not valid UTF-8, it is returned as the original `Vec` in the `Err`
    /// part of the inner result.
    fn recv_string(&self, flags: i32) -> io::Result<result::Result<String, Vec<u8>>> {
        self.recv_string(flags).map_err(|e| e.into())
    }

    /// Receive a multipart message from the socket.
    ///
    /// Note that this will allocate a new vector for each message part; for many applications it
    /// will be possible to process the different parts sequentially and reuse allocations that
    /// way.
    fn recv_multipart(&self, flags: i32) -> io::Result<Vec<Vec<u8>>> {
        self.recv_multipart(flags).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {}
