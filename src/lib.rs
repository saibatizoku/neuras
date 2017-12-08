//! neuras - A high-level API for networking with ØMQ ("zeromq") and tokio.
//! =======================================================================
//!
//! An attempt at having a high-level API on top of ØMQ's awesome foundations,
//! as suggested by
//! "[Features of a Higher-Level API](http://zguide.zeromq.org/page:all#toc74)",
//! by using tokio's reactor and tools.
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
pub extern crate futures;
pub extern crate tokio_core;
pub extern crate url;
pub extern crate zmq;
pub extern crate zmq_tokio;

/// Error handling.
pub mod errors;
// Secure-socket communications.
pub mod secure;
// Useful utilities to deal with ZMQ.
pub mod utils;


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
