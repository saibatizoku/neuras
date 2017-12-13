//! Convenient ZMQ sockets.
//!
//! A high-level socket API that hides regular `zmq::Context` and `zmq::Socket`.
//!
//! Inspired by [zsock](http://czmq.zeromq.org/czmq4-0:zsock).
pub mod errors {
    //! Socket Errors.
    use zmq;
    error_chain! {
        foreign_links {
            Zmq(zmq::Error);
        }
    }
}

use zmq;

use super::initialize::sys_context;
use self::errors::*;

// Socket flags
bitflags! {
    /// Flags for sending/receiving `zmq::Message` on a `zmq::Socket`.
    ///
    /// `SocketFlags::ASYNC` is exactly the same as `ZMQ_DONTWAIT`
    /// `SocketFlags::MULTIPART` is exactly the same as `ZMQ_SNDMORE`
    ///
    /// Default value with bits: `0`
    #[derive(Default)]
    pub struct SocketFlags: i32 {
        const ASYNC = 1;
        const MULTIPART = 2;
        const ASYNC_MULTIPART = Self::ASYNC.bits | Self::MULTIPART.bits;
    }
}

/// Create a new socket given the `zmq::SocketType`
pub fn socket_new(socket_type: zmq::SocketType) -> Result<Socket> {
    let context = sys_context();
    let inner = context.socket(socket_type)?;
    Ok( Socket { inner } )
}

// TODO: use typed endpoints
/// Create a new `zmq::SUB` socket given an endpoint. Default action is `connect`
pub fn socket_new_sub(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::SUB)?;
    let _ = socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PUB` socket given an endpoint. Default action is `bind`
pub fn socket_new_pub(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::PUB)?;
    let _ = socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::REQ` socket given an endpoint. Default action is `connect`
pub fn socket_new_req(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::REQ)?;
    let _ = socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::REP` socket given an endpoint. Default action is `bind`
pub fn socket_new_rep(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::REP)?;
    let _ = socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::DEALER` socket given an endpoint. Default action is `connect`
pub fn socket_new_dealer(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::DEALER)?;
    let _ = socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::ROUTER` socket given an endpoint. Default action is `bind`
pub fn socket_new_router(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::ROUTER)?;
    let _ = socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PUSH` socket given an endpoint. Default action is `connect`
pub fn socket_new_push(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::PUSH)?;
    let _ = socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PULL` socket given an endpoint. Default action is `bind`
pub fn socket_new_pull(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::PULL)?;
    let _ = socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::XSUB` socket given an endpoint. Default action is `connect`
pub fn socket_new_xsub(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::XSUB)?;
    let _ = socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::XPUB` socket given an endpoint. Default action is `bind`
pub fn socket_new_xpub(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::XPUB)?;
    let _ = socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PAIR` socket given an endpoint. Default action is `connect`
pub fn socket_new_pair(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::PAIR)?;
    let _ = socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::STREAM` socket given an endpoint. Default action is `connect`
pub fn socket_new_stream(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::STREAM)?;
    let _ = socket_connect(&socket, endpoint)?;
    Ok(socket)
}

/// Return a reference to the underlying `zmq::Socket`
pub fn socket_resolve(socket: &Socket) -> &zmq::Socket {
    &socket.inner
}

/// Bind a socket to a given endpoint
pub fn socket_bind(socket: &Socket, ep: &str) -> Result<String> {
    let _bind = socket.resolve().bind(ep)?;
    Ok(())
}

/// Connect a socket to a given endpoint
pub fn socket_connect(socket: &Socket, ep: &str) -> Result<()> {
    let _bind = socket.resolve().connect(ep)?;
    Ok(())
}

/// Convenient API around `zmq::Socket`
pub struct Socket {
    inner: zmq::Socket,
}

/// Socket type associated functions
impl Socket {
    /// Create a new socket given the `zmq::SocketType`
    pub fn new(socket_type: zmq::SocketType) -> Result<Socket> {
        socket_new(socket_type)
    }

    // TODO: use typed endpoints
    /// Create a new `zmq::PUB` socket given an endpoint. Default action is `bind`
    pub fn new_pub(endpoint: &str) -> Result<Socket> {
        socket_new_pub(endpoint)
    }

    // TODO: use typed endpoints
    /// Create a new `zmq::SUB` socket given an endpoint. Default action is `connect`
    pub fn new_sub(endpoint: &str) -> Result<Socket> {
        socket_new_sub(endpoint)
    }
}

/// Socket instance methods
impl Socket {
    /// Return a reference to the underlying `zmq::Socket`
    pub fn resolve(&self) -> &zmq::Socket {
        socket_resolve(&self)
    }
    /// Bind a socket to a given endpoint
    pub fn bind(&self, ep: &str) -> Result<()> {
        socket_bind(&self, ep)
    }

    /// Connect a socket to a given endpoint
    pub fn connect(&self, ep: &str) -> Result<()> {
        socket_connect(&self, ep)
    }
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
