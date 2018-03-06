//! Convenient ZMQ sockets.
//!
//! A high-level socket API that hides regular `zmq::Context` and `zmq::Socket`.
//!
//! Inspired by [zsock](http://czmq.zeromq.org/czmq4-0:zsock).
use super::initialize::sys_context;

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

/// Convenient API around `zmq::Socket`
pub struct Socket {
    inner: zmq::Socket,
    is_serverish: bool,
}

/// Socket type associated functions
impl Socket {
    /// Create a new socket given the `zmq::SocketType`
    pub fn new(socket_type: zmq::SocketType) -> Result<Socket, SocketError> {
        socket_new(socket_type)
    }

    // TODO: use typed endpoints
    /// Create a new `zmq::PUB` socket given an endpoint. Default action is `bind`
    pub fn new_pub(endpoint: &str) -> Result<Socket, SocketError> {
        socket_new_pub(endpoint)
    }

    // TODO: use typed endpoints
    /// Create a new `zmq::SUB` socket given an endpoint. Default action is `connect`
    pub fn new_sub(endpoint: &str) -> Result<Socket, SocketError> {
        socket_new_sub(endpoint)
    }

    // TODO: use typed endpoints
    /// Create a new `zmq::PULL` socket given an endpoint.
    pub fn new_pull(endpoint: &str) -> Result<Socket, SocketError> {
        socket_new_pull(endpoint)
    }

    // TODO: use typed endpoints
    /// Create a new `zmq::PUSH` socket given an endpoint.
    pub fn new_push(endpoint: &str) -> Result<Socket, SocketError> {
        socket_new_push(endpoint)
    }
}

/// Socket instance methods
impl Socket {
    /// Return a reference to the underlying `zmq::Socket`
    pub fn resolve(&self) -> &zmq::Socket {
        socket_resolve(self)
    }

    /// Returns `true` if the endpoint is marked for binding to a socket,
    /// it returns `false` if it is marked for connecting to it.
    pub fn is_serverish(&self) -> bool {
        self.is_serverish
    }

    pub fn set_serverish(&mut self, f: bool) {
        self.is_serverish = f;
    }

    /// Run `bind` or `connect` on a socket, depending on what `is_serverish()` returns.
    pub fn plug(&self, endpoint: &str) -> Result<(), SocketError> {
        if self.is_serverish {
            self.bind(endpoint)?;
        } else {
            self.connect(endpoint)?;
        }
        Ok(())
    }

    /// Bind a socket to a given endpoint. Returns the
    /// actual endpoint bound. Useful for unbinding the
    /// socket.
    pub fn bind(&self, ep: &str) -> Result<String, SocketError> {
        socket_bind(self, ep)
    }

    /// Connect a socket to a given endpoint
    pub fn connect(&self, ep: &str) -> Result<(), SocketError> {
        socket_connect(self, ep)
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

// Socket flags
bitflags! {
    /// Flags for sending/receiving `zmq::Message` on a `zmq::Socket`.
    ///
    /// `SocketFlags::ASYNC` is exactly the same as `ZMQ_DONTWAIT`
    /// `SocketFlags::MULTIPART` is exactly the same as `ZMQ_SNDMORE`
    ///
    /// Default value with bits: `0`
    #[derive(Default)]
    struct SocketFlags: i32 {
        const ASYNC = 1;
        const MULTIPART = 2;
        const ASYNC_MULTIPART = Self::ASYNC.bits | Self::MULTIPART.bits;
    }
}

/// Create a new socket given the `zmq::SocketType`
pub fn socket_new(socket_type: zmq::SocketType) -> Result<Socket, SocketError> {
    let context = sys_context();
    let inner = context.socket(socket_type)?;
    let is_serverish = false;
    Ok(Socket {
        inner,
        is_serverish,
    })
}

// TODO: use typed endpoints
/// Create a new `zmq::SUB` socket given an endpoint. Default action is `connect`
pub fn socket_new_sub(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::SUB)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PUB` socket given an endpoint. Default action is `bind`
pub fn socket_new_pub(endpoint: &str) -> Result<Socket, SocketError> {
    let mut socket = Socket::new(zmq::PUB)?;
    socket.set_serverish(true);
    socket.plug(endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::REQ` socket given an endpoint. Default action is `connect`
pub fn socket_new_req(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::REQ)?;
    socket.plug(endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::REP` socket given an endpoint. Default action is `bind`
pub fn socket_new_rep(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::REP)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::DEALER` socket given an endpoint. Default action is `connect`
pub fn socket_new_dealer(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::DEALER)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::ROUTER` socket given an endpoint. Default action is `bind`
pub fn socket_new_router(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::ROUTER)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PUSH` socket given an endpoint. Default action is `connect`
pub fn socket_new_push(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::PUSH)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PULL` socket given an endpoint. Default action is `bind`
pub fn socket_new_pull(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::PULL)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::XSUB` socket given an endpoint. Default action is `connect`
pub fn socket_new_xsub(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::XSUB)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::XPUB` socket given an endpoint. Default action is `bind`
pub fn socket_new_xpub(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::XPUB)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PAIR` socket given an endpoint. Default action is `connect`
pub fn socket_new_pair(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::PAIR)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::STREAM` socket given an endpoint. Default action is `connect`
pub fn socket_new_stream(endpoint: &str) -> Result<Socket, SocketError> {
    let socket = Socket::new(zmq::STREAM)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

/// Return a reference to the underlying `zmq::Socket`
pub fn socket_resolve(socket: &Socket) -> &zmq::Socket {
    &socket.inner
}

/// Bind a socket to a given endpoint
pub fn socket_bind(socket: &Socket, ep: &str) -> Result<String, SocketError> {
    socket.resolve().bind(ep)?;
    let endpoint = match socket.resolve().get_last_endpoint()? {
        Ok(res) => res,
        Err(e) => return Err(SocketError::Endpoint(e)),
    };
    Ok(endpoint)
}

/// Connect a socket to a given endpoint
pub fn socket_connect(socket: &Socket, ep: &str) -> Result<(), SocketError> {
    socket.resolve().connect(ep)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zmq;

    #[test]
    fn create_new_socket_from_socket_type() {
        let socket = socket_new(zmq::PUB).unwrap();
        assert_eq!(socket.resolve().get_socket_type(), Ok(zmq::PUB));
    }

    #[test]
    fn create_new_pub_socket() {
        let socket = socket_new_pub("inproc://pub_test").unwrap();
        assert_eq!(socket.resolve().get_socket_type(), Ok(zmq::PUB));
        let last_bound_ep = socket.resolve().get_last_endpoint().unwrap();
        assert_eq!(last_bound_ep, Ok("inproc://pub_test".to_string()));
    }

    #[test]
    fn create_new_sub_socket() {
        let socket = socket_new_sub("inproc://pub_test").unwrap();
        assert_eq!(socket.resolve().get_socket_type(), Ok(zmq::SUB));
    }

    #[test]
    fn socket_flags_default_to_empty_bitmask() {
        let flags: SocketFlags = Default::default();
        assert!(flags.is_empty());
        assert_eq!(flags.bits(), 0);
    }

    #[test]
    fn socket_flags_async_is_zmq_dontwait() {
        let flag = SocketFlags::ASYNC;
        assert_eq!(flag.bits(), zmq::DONTWAIT);
    }

    #[test]
    fn socket_flags_multipart_is_zmq_sndmore() {
        let flag = SocketFlags::MULTIPART;
        assert_eq!(flag.bits(), zmq::SNDMORE);
    }

    #[test]
    fn socket_flags_async_multipart_is_zmq_sndmore() {
        let flags = SocketFlags::ASYNC_MULTIPART;
        assert_eq!(flags.bits(), zmq::DONTWAIT | zmq::SNDMORE);
    }
}
