//! Convenient ZMQ sockets.
//!
//! A high-level socket API that hides regular `zmq::Context` and `zmq::Socket`.
//!
//! Inspired by [zsock](http://czmq.zeromq.org/czmq4-0:zsock).
//!
//! # Examples
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
//!     neuras::init();
//!
//!     println!("sender starting");
//!
//!     // Create a 'sender' socket, using the usual `rust-zmq`
//!     // `zmq::SocketType`. Note there is no context declaration
//!     let sender = Socket::new(zmq::PAIR)?;
//!     // Bind it, the usuall way.
//!     sender.bind("inproc://socket_example")?;
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
//!     sender.resolve().send("hi", zmq::SNDMORE)?;
//!     println!("sender sent: {:?}", "hi");
//!     sender.resolve().send("world", 0)?;
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
//!     run_synchronous_program().unwrap();
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
//!     sender.set_serverish(true);
//!     sender.plug("inproc://socket_example")?;
//!
//!     // Use a convenience function to build a connected PAIR.
//!     // It automatically calls the socket's `connect` function.
//!     println!("client starting");
//!     let client = Socket::new(zmq::PAIR)?;
//!     client.plug("inproc://socket_example")?;
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
//!             sender_socket.send("hi", zmq::SNDMORE)?;
//!             println!("server sent: {:?}", "hi");
//!
//!             sender_socket.send("world", 0)?;
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
//!     neuras::init();
//!     run_polling_program().unwrap();
//! }
//! ```
use self::errors::*;
use super::initialize::sys_context;

use std::io;
use std::result;
use zmq;

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

#[cfg(feature = "async-mio")]
#[path = "socket_mio.rs"]
pub mod mio;
#[cfg(feature = "async-tokio")]
#[path = "socket_tokio.rs"]
pub mod tokio;

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
    pub fn plug(&self, endpoint: &str) -> Result<()> {
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
    pub fn bind(&self, ep: &str) -> Result<String> {
        socket_bind(self, ep)
    }

    /// Connect a socket to a given endpoint
    pub fn connect(&self, ep: &str) -> Result<()> {
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
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PUB` socket given an endpoint. Default action is `bind`
pub fn socket_new_pub(endpoint: &str) -> Result<Socket> {
    let mut socket = Socket::new(zmq::PUB)?;
    socket.set_serverish(true);
    socket.plug(endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::REQ` socket given an endpoint. Default action is `connect`
pub fn socket_new_req(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::REQ)?;
    socket.plug(endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::REP` socket given an endpoint. Default action is `bind`
pub fn socket_new_rep(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::REP)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::DEALER` socket given an endpoint. Default action is `connect`
pub fn socket_new_dealer(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::DEALER)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::ROUTER` socket given an endpoint. Default action is `bind`
pub fn socket_new_router(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::ROUTER)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PUSH` socket given an endpoint. Default action is `connect`
pub fn socket_new_push(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::PUSH)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PULL` socket given an endpoint. Default action is `bind`
pub fn socket_new_pull(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::PULL)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::XSUB` socket given an endpoint. Default action is `connect`
pub fn socket_new_xsub(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::XSUB)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::XPUB` socket given an endpoint. Default action is `bind`
pub fn socket_new_xpub(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::XPUB)?;
    socket_bind(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::PAIR` socket given an endpoint. Default action is `connect`
pub fn socket_new_pair(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::PAIR)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

// TODO: use typed endpoints
/// Create a new `zmq::STREAM` socket given an endpoint. Default action is `connect`
pub fn socket_new_stream(endpoint: &str) -> Result<Socket> {
    let socket = Socket::new(zmq::STREAM)?;
    socket_connect(&socket, endpoint)?;
    Ok(socket)
}

/// Return a reference to the underlying `zmq::Socket`
pub fn socket_resolve(socket: &Socket) -> &zmq::Socket {
    &socket.inner
}

/// Bind a socket to a given endpoint
pub fn socket_bind(socket: &Socket, ep: &str) -> Result<String> {
    socket.resolve().bind(ep)?;
    let endpoint = match socket.resolve().get_last_endpoint()? {
        Ok(res) => res,
        Err(_) => bail!("trouble getting last bound endpoint"),
    };
    Ok(endpoint)
}

/// Connect a socket to a given endpoint
pub fn socket_connect(socket: &Socket, ep: &str) -> Result<()> {
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
