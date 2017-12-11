//! neuras - A high-level API for networking with ØMQ ("zeromq") and tokio.
//! =======================================================================
//!
//! An attempt at having a high-level API on top of ØMQ's awesome foundations,
//! as suggested by
//! "[Features of a Higher-Level API](http://zguide.zeromq.org/page:all#toc74)",
//! by using tokio's reactor and tools.
#![recursion_limit = "1024"]

#[macro_use]
extern crate bitflags;
extern crate chrono;
#[macro_use]
extern crate error_chain;
pub extern crate futures;
pub extern crate tokio_core;
pub extern crate url;
pub extern crate zmq;
pub extern crate zmq_tokio;

// Millisecond clocks and delays.
pub mod clock;
/// Error handling.
pub mod errors;
// Security for socket communications.
pub mod security;
// Useful utilities to deal with ZMQ.
pub mod utils;

bitflags! {
    /// Typesafe flags for sending/receiving `zmq::Message` on a `zmq::Socket`.
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

#[cfg(test)]
mod tests {
    use super::*;
    use zmq;

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
