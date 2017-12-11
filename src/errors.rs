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
