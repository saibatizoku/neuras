// Modeled after tests/smoke.rs from zmq-mio, which is modeled after
// tests/udp.rs from tokio-core.
//
// This example is mostly to show how to create a REQ-REP service
// using ZMQ sockets as the underlying transport.
//
// The REQ service is a ASYNCHRONOUS ZMQ socket running within a tokio reactor.
//
// The REP service is a SYNCHRONOUS ZMQ socket running on a regular
// loop. This service could be anywhere in the client's network.
//
// It is mostly a proof-of-concept exercise.
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate neuras;
extern crate tokio_core;
extern crate zmq;
extern crate zmq_tokio;

use std::io;

use neuras::security::{secure_client_socket, secure_server_socket};
use neuras::errors::*;
use futures::{stream, Future, Sink, Stream};
use tokio_core::reactor::Core;
use zmq_tokio::{convert_into_tokio_socket, Socket};

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

//const SOCKET_ADDRESS: &'static str = "tcp://127.0.0.1:3294";
const SOCKET_ADDRESS: &'static str = "tcp://127.0.0.1:5657";
//const SOCKET_ADDRESS: &'static str = "inproc://tokio-req-client";

// A stream of client requests that print responses to stdout.
// The client sends a request and gathers a two-part message.
//
// If the two parts are not read, the server will block forever.
fn stream_client(
    req: Socket,
    count: u64,
) -> Box<futures::Future<Item = (), Error = io::Error> + std::marker::Send + 'static> {
    Box::new(
        stream::iter_result((0..count).map(Ok))
            .fold(
                req.framed().split(),
                move |(client_writer, client_reader), i| {
                    let msg_string = format!("Hello {}", i);
                    let msg = zmq::Message::from_slice(msg_string.as_bytes());

                    client_writer.send(msg).and_then(move |client_writer| {
                        println!("REQuesting: {}", msg_string);
                        let show_reply = client_reader
                            .into_future()
                            .and_then(move |(reply, rest)| {
                                //println!("full reply: {:?}", reply);
                                if let Some(msg) = reply {
                                    match msg.as_str() {
                                        Some(s) => println!("REsPonded: {}", s),
                                        _ => {}
                                    }
                                }
                                Ok(rest)
                            })
                            .and_then(move |rest| {
                                rest.into_future().and_then(move |(reply, rest)| {
                                    if let Some(msg) = reply {
                                        match msg.as_str() {
                                            Some(s) => println!("REsPonded: {}", s),
                                            _ => {}
                                        }
                                    }
                                    Ok(rest)
                                })
                            });
                        show_reply
                            .map(|client_reader| (client_writer, client_reader))
                            .map_err(|(e, _)| e)
                    })
                },
            )
            .map(|_| {}),
    )
}

fn run_main() -> Result<()> {
    // `zmq::Context` to be shared by other `zmq::Socket` connections.
    // The context IS thread-safe.
    //
    // `zmq::Socket` IS NOT thread-safe, and must be instantiated within
    // the thread where it will exist.
    let ctx = zmq::Context::new();

    // Make a Curve key-pair for our server.
    let server_keys = zmq::CurveKeyPair::new().unwrap();
    // clone the public key before it is moved by a thread.
    let serverkey = server_keys.public_key.clone();

    // Receiver setup
    // --------------
    // This is a standard ZMQ socket running on a spawned thread,
    // listening on a loop.
    let server_ctx = ctx.clone();
    let server = std::thread::spawn(move || {
        // Create a `zmq::Socket` with the `zmq::REP` socket-type.
        let rep = t!(server_ctx.socket(zmq::REP));
        let _chiper = secure_server_socket(&rep, &server_keys).unwrap();

        // Connect the `zmq_tokio::Socket` to the given endpoint.
        let _connect = t!(rep.bind(SOCKET_ADDRESS));
        // Create an ugly counter that will help us exit the loop at some
        // future point. Real applications should implement better mechanisms
        // that are fully tested.
        let mut cnt = 0;
        // Reusable `zmq::Message` to spare memory allocation.
        let mut msg = zmq::Message::new();
        loop {
            // Step 1: REP listens for an incoming message
            //
            // Notice that ZMQ is blocking until it receives something.
            let _read = rep.recv(&mut msg, 0).expect("couldn't read request");
            let m = msg.as_str().unwrap();
            println!("server processing: {:?}", m);
            // Step 2: REP writes back a response
            //
            // Notice that ZMQ is blocking until it sends something.
            //
            // This example makes use of sending a multipart message that simply
            // says "bye" on the last frame.
            let _reply = rep.send(m, zmq::SNDMORE).expect("Couldn't send data");
            let _reply = rep.send("bye", 0).expect("Couldn't send bye");

            // Our ugly loop control mechanism. Make your own, test it, and, why not?,
            // catch system signals for `Ctrl-C`.
            cnt += 1;
            if cnt == 10 {
                break;
            }
        }
    });

    // Sender setup
    // --------------
    let client_ctx = ctx.clone();
    let client: std::thread::JoinHandle<Result<()>> = std::thread::spawn(move || {
        // Tokio reactor core that will run our application.
        let mut reactor = Core::new().unwrap();
        // Get a handle to the reactor.
        let handle = reactor.handle();

        // Create a `zmq::Socket` with the `zmq::REQ` socket-type.
        // The socket can be configured as usual before converting it into
        // a `zmq_tokio::Socket`.
        let req_socket = t!(client_ctx.socket(zmq::REQ));
        let client_keys = zmq::CurveKeyPair::new().unwrap();
        let _chiper = secure_client_socket(&req_socket, &serverkey, &client_keys).unwrap();

        // Create a `mut zmq_tokio::Socket` from the `zmq::Socket` and the
        // reactor handle.
        let req = t!(convert_into_tokio_socket(req_socket, &handle));

        // Connect the `zmq_tokio::Socket` to the given endpoint.
        let _connect = t!(req.connect(SOCKET_ADDRESS));

        // Create a `Future` with the result from our client streams.
        let client = stream_client(req, 10);
        // Make the future happen.
        let c = reactor.run(client);
        if let Err(e) = c {
            println!("throuble with client reactor: {:?}", e);
            bail!(ErrorKind::Msg("client reactor error".to_string()));
        }
        c.unwrap();
        Ok(())
    });

    if let Err(e) = client.join() {
        println!("throuble with joining thread: {:?}", e);
        bail!("client thread error");
    }

    let _ = server.join().expect("throuble with joining thread");
    Ok(())
}

quick_main!(run_main);
