//! This example uses `CipherSocketBuilder` to create a `CipherReceiver`, and a
//! `CipherSender`. These are ZMQ sockets that act as server-client through a
//! given endpoint.
//!
//! These are useful for inter-connecting sockets securely using `ipc://`,
//! `tcp://`, or `udp://`. The `inproc://` scheme can only be used for sockets
//! that share the same ZMQ context.
#[macro_use]
extern crate error_chain;
extern crate neuras;
extern crate zmq;

use neuras::security::{CipherReceiver, CipherSender, CipherSocketBuilder};
use neuras::security::errors::*;
use zmq::Message;

// Utility for creating `PAIR` sockets with a common endpoint pre-configured.
fn create_cipher_pair(endpoint: &str) -> Result<(CipherSender, CipherReceiver)> {
    // Builder to create cipher sockets. Takes `Option<zmq::Context>` as the
    // only argument.
    let socket_builder = CipherSocketBuilder::new()?;

    // Bind the receiver socket to the endpoint
    println!("binding server to {:?}", &endpoint);
    let receiver = socket_builder.receiver(zmq::PAIR, endpoint)?;
    let _bind = receiver.bind()?;

    // Get the last endpoint the receiver was bound to. ZMQ sockets can be
    // bound to multiple endpoints, such as `tcp://127.0.0.1:*` which are
    // dynamically assigned from free ports. Calling `get_last_endpoint` is
    // needed to specify the exact endpoint for the senders to connect to.
    let ep = receiver.get_last_endpoint()?;

    // Connect the sender socket to the endpoint
    println!("connecting client to {:?}", &ep);
    let sender = socket_builder.sender(zmq::PAIR, &ep)?;
    let _connect = sender.connect(&receiver.public_key())?;

    Ok((sender, receiver))
}

// Run the example
fn run_code() -> Result<()> {
    // Define the endpoint. Supports the following schemes:
    // - 'tcp://'
    // - 'udp://'
    // - 'ipc://'
    // -'inproc://' (ZMQ-specific for inter-thread communication)
    //
    // NOTE: 'inproc://' sockets completely ignore any Curve KeyPair
    // settings. Given their inter-thread nature, it is not considered
    // to be required (..I'm guessing..).
    let endpoint = "ipc://secure";

    // While zmq::Socket offers convenience methods for handling &[u8], &str,
    // and Strings, we create a zmq::Message that will be reused throughout,
    // saving memory allocation and speeding up our program while receiving
    // incoming bytes.
    let mut msg = Message::new();

    // Build the sender and the receiver, each on a separate thread, sharing the
    // same zmq::Context, and thus enabling the possibility for inter-process
    // communication.
    let (sender, receiver) = create_cipher_pair(endpoint)?;

    // The typical REQ-REP pattern is as follows:
    //
    // 1. sender sends REQ
    sender.send("foo", 0)?;

    // 2. receiver gets REQ
    receiver.recv(&mut msg, 0)?;

    // 3. receiver processes REQ
    {
        assert_eq!(&msg[..], b"foo");
        assert_eq!(msg.as_str(), Some("foo"));
        println!("this is it {0}", msg.as_str().unwrap());
        assert_eq!(format!("{:?}", msg), "[102, 111, 111]");
    }

    // 4. receiver responds with REP
    receiver.send("baraboom", 0)?;

    // 5. sender gets REP
    sender.recv(&mut msg, 0)?;

    // 6. sender processes REP
    {
        assert_eq!(&msg[..], b"baraboom");
        println!("...{}", msg.as_str().unwrap());
    }

    // 7. cleanup connections before leaving
    receiver.disconnect()?;
    sender.disconnect()?;

    Ok(())
}

// Macro for `fn main()` that catches error-chains
quick_main!(run_code);
