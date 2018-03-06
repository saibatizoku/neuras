//! One-point stop for error-handling.
//!
//! Error handling is done using `error-chain` to link module-level
//! errors. If you are only using specific modules from this crate,
//! it might be better to use module-level errors.
//!
//! Check the documentation of the module that you are using to learn
//! more about the errors it handles.
use std::io;

use zmq;

use super::security;

error_chain! {
    errors {
        AddressParse {
            description ("could not parse address")
        }
        ConfigParse {
            description ("could not parse configuration file")
        }
        Neurotic {
            description ("our network has gone neurotic")
        }
    }
    links {
        Security(security::errors::Error, security::errors::ErrorKind);
    }
    foreign_links {
        Io(io::Error);
        Zmq(zmq::Error);
    }
}
