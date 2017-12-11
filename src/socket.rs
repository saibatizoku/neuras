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

use self::errors::*;

#[cfg(test)]
mod tests {
    use super::*;
    use zmq;

    #[test]
    fn create_new_socket_from_socket_type() {
        let socket = Socket::new(zmq::PUB, None).unwrap();
        assert_eq!(socket.socket_type, zmq::PUB);
    }

    #[test]
    fn create_new_pub_socket() {
        let socket = Socket::new_pub("inproc://pub_test").unwrap();
        assert_eq!(socket.socket_type, zmq::PUB);
    }
}
