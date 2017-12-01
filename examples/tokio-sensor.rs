// Modeled after tests/smoke.rs from zmq-mio, which is modeled after
// tests/udp.rs from tokio-core.
//
// This example is mostly to show how to create a REQ-REP service
// using ZMQ sockets as the underlying transport.
//
// It is mostly a proof-of-concept exercise.
extern crate futures;
extern crate tokio_core;
extern crate zmq;
extern crate zmq_tokio;

use std::io;

use futures::{stream, Future, Sink, Stream};
use tokio_core::reactor::Core;
use zmq_tokio::Socket;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
    })
}

const SOCKET_ADDRESS: &'static str = "tcp://127.0.0.1:3294";

fn stream_server(
    rep: Socket,
    count: u64,
) -> Box<futures::Future<Item = (), Error = io::Error> + std::marker::Send + 'static> {
    println!("server started");
    let (responses, requests) = rep.framed().split();
    Box::new(
        requests
            .take(count)
            .fold(responses, |responses, mut request| {
                // FIXME: multipart send support missing, this is a crude hack
                println!("REQ: {:?}", String::from_utf8(request[0].clone()).unwrap());
                let mut part0 = None;
                for part in request.drain(0..1) {
                    part0 = Some(part);
                    break;
                }
                responses.send(part0.unwrap())
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
            .fold(req.framed().split(), move |(requests, responses), i| {
                requests
                    .send(format!("Hello {}", i).into())
                    .and_then(move |requests| {
                        println!("request sent!");
                        let show_reply = responses.into_future().and_then(move |(reply, rest)| {
                            println!(
                                "REP: {:?}",
                                String::from_utf8(reply.unwrap()[0].clone()).unwrap()
                            );
                            Ok(rest)
                        });
                        show_reply
                            .map(|responses| (requests, responses))
                            .map_err(|(e, _)| e)
                    })
            })
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

    // Create a `zmq_tokio::Socket` from the `zmq::Socket` and the
    // reactor handle.
    let mut rep = t!(Socket::new(zmq_rep_socket, &handle));

    // Bind the `zmq_tokio::Socket` to the given endpoint.
    let _bind = t!(rep.bind(SOCKET_ADDRESS));

    let client = std::thread::spawn(move || {
        let mut l = Core::new().unwrap();
        let handle = l.handle();

        // Sender setup
        // --------------
        // Create a `zmq::Socket` with the `zmq::REQ` socket-type.
        // The socket can be configured as usual before converting it into
        // a `zmq_tokio::Socket`.
        let req_socket = t!(ctx.socket(zmq::REQ));

        // Create a `mut zmq_tokio::Socket` from the `zmq::Socket` and the
        // reactor handle.
        let mut req = t!(Socket::new(req_socket, &handle));

        // Connect the `zmq_tokio::Socket` to the given endpoint.
        let _connect = t!(req.connect(SOCKET_ADDRESS));

        let client = stream_client(req, 10);
        l.run(client).unwrap();
    });

    let server = stream_server(rep, 10);
    l.run(server).unwrap();
    client.join().unwrap();
}
