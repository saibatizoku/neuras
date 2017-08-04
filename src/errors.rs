use std::io;
use zmq;

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
    foreign_links {
        Io(io::Error);
        Zmq(zmq::Error);
    }
}
