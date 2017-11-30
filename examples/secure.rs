#[macro_use]
extern crate error_chain;
extern crate neuras;
extern crate zmq;

use neuras::secure::{CipherReceiver, CipherSender, CipherSocketBuilder};
use neuras::secure::errors::*;
use zmq::{Context, Message};

// Utility for creating `PAIR` sockets with a common endpoint
fn create_socket_pair(
    endpoint: &str,
    context: Option<Context>,
) -> Result<(CipherSender, CipherReceiver)> {
    let socket_builder = CipherSocketBuilder::new(context)?;

    let receiver = socket_builder.receiver(zmq::PAIR, endpoint)?;
    let _bind = receiver.bind()?;

    let ep = receiver.get_last_endpoint()?;

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

    // ZMQ needs a context to run the socket. It is NOT an event-loop reactor.
    let net_ctx = Context::new();

    // While zmq::Socket offers convenience methods for handling &[u8], &str,
    // and Strings, we create a zmq::Message that will be reused throughout,
    // saving memory allocation and speeding up our program while receiving
    // incoming bytes.
    let mut msg = Message::new();

    // Build the sender and the receiver, each on a separate thread, sharing the
    // same zmq::Context, and thus enabling the possibility for inter-process
    // communication.
    let (sender, receiver) = create_socket_pair(endpoint, Some(net_ctx))?;

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
