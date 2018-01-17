// Modeled after tests/smoke.rs from zmq-mio, which is modeled after
// tests/udp.rs from tokio-core.
//
// This example is mostly to show how to create an asyc REQ-REP service
// using ZMQ sockets as the underlying transport.
//
// It is mostly a proof-of-concept exercise.
//
// Both REQ and REP are ZMQ sockets running within a tokio reactor.
extern crate futures;
extern crate neuras;
extern crate tokio_core;
extern crate zmq;
extern crate zmq_tokio;

use std::io;
use std::ops::Deref;

use neuras::security::{secure_client_socket, secure_server_socket};
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

fn stream_server(
    rep: Socket,
    count: u64,
) -> Box<futures::Future<Item = (), Error = io::Error> + std::marker::Send + 'static> {
    println!("server started");
    let (server_tx, server_rx) = rep.framed().split();
    Box::new(
        server_rx
            .take(count)
            .fold(server_tx, |server_tx, request| {
                // FIXME: multipart send support missing, this is a crude hack
                server_tx.send(request.deref().into())
            })
            .map(|_| {}),
    )
}

fn stream_client(
    req: Socket,
    count: u64,
) -> Box<futures::Future<Item = (), Error = io::Error> + std::marker::Send + 'static> {
    Box::new(
        stream::iter_result((0..count).map(Ok))
            .fold(
                req.framed().split(),
                move |(client_writer, client_reader), i| {
                    let msg = format!("Hello {}", i);
                    client_writer
                        .send(msg.as_str().into())
                        .and_then(move |client_writer| {
                            println!("REQuesting: {}", msg);
                            let show_reply =
                                client_reader.into_future().and_then(move |(reply, rest)| {
                                    if let Some(msg) = reply {
                                        let _ = match msg.as_str() {
                                            Some(s) => println!("REsPonded: {}", s),
                                            _ => {},
                                        };
                                    }
                                    Ok(rest)
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

fn main() {
    // Tokio reactor core that will run our application.
    let mut l = Core::new().unwrap();
    // Get a handle to the reactor.
    let handle = l.handle();

    // `zmq::Context` to be shared by other `zmq::Socket` connections.
    // The context IS thread-safe.
    //
    // `zmq::Socket` IS NOT thread-safe, and must be instantiated within
    // the thread where it will exist.
    let ctx = zmq::Context::new();

    // Receiver setup
    // --------------
    // Create a `zmq::Socket` with the `zmq::REP` socket-type.
    // The socket can be configured as usual before converting it into
    // a `zmq_tokio::Socket`.
    let zmq_rep_socket = t!(ctx.socket(zmq::REP));
    let server_keys = zmq::CurveKeyPair::new().unwrap();
    let _chiper = secure_server_socket(&zmq_rep_socket, &server_keys).unwrap();
    let serverkey = server_keys.public_key.clone();

    // Create a `zmq_tokio::Socket` from the `zmq::Socket` and the
    // reactor handle.
    let rep = t!(convert_into_tokio_socket(zmq_rep_socket, &handle));

    let client_ctx = ctx.clone();
    let client = std::thread::spawn(move || {
        let mut l = Core::new().unwrap();
        let handle = l.handle();

        // Sender setup
        // --------------
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

        let client = stream_client(req, 10);
        l.run(client).unwrap();
    });

    // Bind the `zmq_tokio::Socket` to the given endpoint.
    let _bind = t!(rep.bind(SOCKET_ADDRESS));
    let server = stream_server(rep, 10);
    l.run(server).unwrap();
    client.join().unwrap();
}

#[test]
fn tokio_test() {
    assert!(true);
}
