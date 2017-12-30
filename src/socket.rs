//! Convenient ZMQ sockets.
//!
//! A high-level socket API that hides regular `zmq::Context` and `zmq::Socket`.
//!
//! Inspired by [zsock](http://czmq.zeromq.org/czmq4-0:zsock).
//!
//! #Examples
//!
//! ## A Good Ol' Synchronous (Blocking) Example
//!
//! This example shows how to run ZMQ sockets like you're used to.
//!
//! ```
//! extern crate neuras;
//! extern crate zmq;
//!
//! use std::thread;
//!
//! use neuras::{init, Socket};
//! use neuras::socket::socket_new_pair;
//! use neuras::errors::*;
//!
//! fn run_synchronous_program() -> Result<()> {
//!     /// RUN THIS ALWAYS FIRST AND ON THE MAIN THREAD.
//!     /// If you don't, beware.... there be monsters here.
//!     ///
//!     /// This initializes a global context that can be
//!     /// used to create `inproc` connections between any
//!     /// socket created in our program. That's a good thing.
//!     let _ = neuras::init();
//!
//!     println!("sender starting");
//!
//!     // Create a 'sender' socket, using the usual `rust-zmq`
//!     // `zmq::SocketType`. Note there is no context declaration
//!     let sender = Socket::new(zmq::PAIR)?;
//!     // Bind it, the usuall way.
//!     let _ = sender.bind("inproc://socket_example")?;
//!     println!("client starting");
//!
//!     // Create a 'client' socket, using the low-level functions
//!     // available in `neuras::socket::*`. In this case, the
//!     // socket is *connected* to the endpoint by default.
//!     let client = socket_new_pair("inproc://socket_example").unwrap();
//!
//!     // Sequencial, blocking calls. This example sends a multipart
//!     // message. ZMQ will not send data until the final part of the
//!     // message is sent WITHOUT a `zmq::SNDMORE` flag. This typically
//!     // means setting flags to `0` or, `zmq::DONTWAIT` when using
//!     // async patterns.
//!     let _ = sender.resolve().send("hi", zmq::SNDMORE)?;
//!     println!("sender sent: {:?}", "hi");
//!     let _ = sender.resolve().send("world", 0)?;
//!     println!("sender sent: {:?}", "world");
//!
//!     // The multipart message was sent, now we receive it.
//!     // We can resolve to use the `zmq::Socket` directly,
//!     // and get the message manually. This is good if you
//!     // want to react depending on any specific message.
//!     let msg = client.resolve().recv_msg(0).unwrap();
//!     assert_eq!(msg.as_str(), Some("hi"));
//!
//!     // ZMQ will have received the whole multipart if it
//!     // has received the first part. That is, the socket
//!     // will be ready for reading only after the whole
//!     // multipart message is received.
//!
//!     // We check the socket to verify there is more of the
//!     // message that we have to receive.
//!     assert_eq!(client.resolve().get_rcvmore(), Ok(true));
//!
//!     let msg = client.resolve().recv_msg(0).unwrap();
//!     assert_eq!(msg.as_str(), Some("world"));
//!
//!     // We check the socket to verify there is no more of the
//!     // message that we have to receive.
//!     assert_eq!(client.resolve().get_rcvmore(), Ok(false));
//!
//!     Ok(())
//! }
//!
//! fn main () {
//!     let _run = run_synchronous_program().unwrap();
//! }
//! ```
//!
//! ## A Good Ol' Polling Example
//!
//! This example shows how to poll ZMQ sockets like you're used to.
//!
//! ```
//! extern crate neuras;
//! extern crate zmq;
//!
//! use std::thread;
//!
//! use neuras::{init, Socket};
//! use neuras::socket::socket_new_pair;
//! use neuras::errors::*;
//!
//! fn run_polling_program() -> Result<()> {
//!
//!     // Build a new socket type, flag it as "serverish",
//!     // and finally "plug" the socket to the endpoint.
//!     // When a socket is marked as "serverish", the socket's
//!     // `bind` function is called.
//!     println!("server starting");
//!     let mut sender = Socket::new(zmq::PAIR)?;
//!     let _ = sender.set_serverish(true);
//!     let _ = sender.plug("inproc://socket_example")?;
//!
//!     // Use a convenience function to build a connected PAIR.
//!     // It automatically calls the socket's `connect` function.
//!     println!("client starting");
//!     let client = Socket::new(zmq::PAIR)?;
//!     let _ = client.plug("inproc://socket_example")?;
//!
//!     // Get a handle to the underlying `zmq::Socket`
//!     let sender_socket = sender.resolve();
//!     let client_socket = client.resolve();
//!
//!     let mut continue_flag = true;
//!
//!     let mut items = [
//!         sender_socket.as_poll_item(zmq::POLLOUT),
//!         client_socket.as_poll_item(zmq::POLLIN),
//!     ];
//!
//!     while continue_flag {
//!
//!         zmq::poll(&mut items, -1).expect("failed polling");
//!
//!         if items[0].is_writable() {
//!             // Sequencial, blocking calls. This example sends a multipart
//!             // message. ZMQ will not send data until the final part of the
//!             // message is sent with a `0` flag.
//!             let _ = sender_socket.send("hi", zmq::SNDMORE)?;
//!             println!("server sent: {:?}", "hi");
//!
//!             let _ = sender_socket.send("world", 0)?;
//!             println!("server sent: {:?}", "world");
//!         }
//!
//!         if items[1].is_readable() {
//!             // The multipart message was sent, now we receive it.
//!             // We can resolve to use the `zmq::Socket` directly,
//!             // and get the message manually. This is good if you
//!             // want to react depending on any specific message.
//!             let msg = client_socket.recv_msg(0).unwrap();
//!             assert_eq!(msg.as_str(), Some("hi"));
//!
//!             // ZMQ will have received the whole multipart if it
//!             // has received the first part. That is, the socket
//!             // will be ready for reading only after the whole
//!             // multipart message is received.
//!
//!             // We check the socket to verify there is more of the
//!             // message that we have to receive.
//!             assert_eq!(client_socket.get_rcvmore(), Ok(true));
//!
//!             let msg = client_socket.recv_msg(0).unwrap();
//!             assert_eq!(msg.as_str(), Some("world"));
//!
//!             // We check the socket to verify there is no more of the
//!             // message that we have to receive.
//!             assert_eq!(client_socket.get_rcvmore(), Ok(false));
//!             continue_flag = false;
//!         }
//!     }
//!
//!     Ok(())
//! }
//!
//! fn main () {
//!     /// RUN THIS ALWAYS FIRST AND ON THE MAIN THREAD.
//!     /// If you don't, beware.... there be monsters here.
//!     let _ = neuras::init();
//!     let _run = run_polling_program().unwrap();
//! }
//! ```
pub mod errors {
    //! Socket Errors.
    use std::io;
    use zmq;
    error_chain! {
        foreign_links {
            Io(io::Error);
            Zmq(zmq::Error);
        }
    }
}

use tokio_core::reactor::Handle;
use zmq;
use zmq_tokio;

use initialize::sys_context;

use self::errors::*;

/// Convenient API around `zmq::Socket`
pub struct Socket {
    inner: zmq::Socket,
    is_serverish: bool,
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

    // TODO: use typed endpoints
    /// Create a new `zmq::PULL` socket given an endpoint.
    pub fn new_pull(endpoint: &str) -> Result<Socket> {
        socket_new_pull(endpoint)
    }

    // TODO: use typed endpoints
    /// Create a new `zmq::PUSH` socket given an endpoint.
    pub fn new_push(endpoint: &str) -> Result<Socket> {
        socket_new_push(endpoint)
    }
}

/// Socket instance methods
impl Socket {
    /// Return a `zmq_tokio::Socket`. Consumes `self`.
    pub fn tokio(self, handle: &Handle) -> Result<zmq_tokio::Socket> {
        let socket = zmq_tokio::Socket::new(self.inner, handle)?;
        Ok(socket)
    }

    /// Return a reference to the underlying `zmq::Socket`
    pub fn resolve(&self) -> &zmq::Socket {
        socket_resolve(&self)
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
    pub fn plug(&self, endpoint: &str) -> Result<()> {
        if self.is_serverish {
            let _ = self.bind(endpoint)?;
        } else {
            let _ = self.connect(endpoint)?;
        }
        Ok(())
    }

    /// Bind a socket to a given endpoint. Returns the
    /// actual endpoint bound. Useful for unbinding the
    /// socket.
    pub fn bind(&self, ep: &str) -> Result<String> {
        socket_bind(&self, ep)
    }

    /// Connect a socket to a given endpoint
    pub fn connect(&self, ep: &str) -> Result<()> {
        socket_connect(&self, ep)
    }
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
    let is_serverish = false;
    Ok(Socket {
        inner,
        is_serverish,
    })
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
    let mut socket = Socket::new(zmq::PUB)?;
    let _ = socket.set_serverish(true);
    let _ = socket.plug(endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::REQ` socket given an endpoint. Default action is `connect`
pub fn socket_new_req(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::REQ)?;
    let _ = socket.plug(endpoint)?;
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
    let endpoint = match socket.resolve().get_last_endpoint()? {
        Ok(res) => res,
        Err(_) => bail!("trouble getting last bound endpoint"),
    };
    Ok(endpoint)
}

/// Connect a socket to a given endpoint
pub fn socket_connect(socket: &Socket, ep: &str) -> Result<()> {
    let _bind = socket.resolve().connect(ep)?;
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
