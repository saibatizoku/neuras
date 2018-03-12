extern crate futures;
extern crate neuras;
extern crate tokio_core;
extern crate zmq;

use neuras::socket::tokio::TokioSocket;
use futures::Future;
use tokio_core::reactor::Core;

macro_rules! t {
    ($e: expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
        }
    };
}

const SOCKET_ADDRESS: &str = "tcp://127.0.0.1:5657";

fn main() {
    let mut reactor = Core::new().unwrap();
    let handle = reactor.handle();

    let ctx = zmq::Context::new();

    // Receiver setup
    let rep_socket = t!(ctx.socket(zmq::REP));
    t!(rep_socket.bind(SOCKET_ADDRESS));
    let server: TokioSocket = (rep_socket, &handle).into();

    // Sender setup
    // --------------
    let req_socket = t!(ctx.socket(zmq::REQ));
    t!(req_socket.connect(SOCKET_ADDRESS));
    let client: TokioSocket = (req_socket, &handle).into();

    // We reuse a message throught
    let mut msg = zmq::Message::new();

    println!("------------------------------");
    println!("REQ-REP with the tokio reactor");
    println!("------------------------------");
    {
        let client_send = client.send("hello-async", 0);
        let server_recv = server.recv(&mut msg, 0);
        let client_send_server_recv = client_send.and_then(|_| server_recv);
        reactor.run(client_send_server_recv).unwrap();
    }
    println!("REQ: {}", msg.as_str().unwrap());

    {
        let server_send = server.send("world-async", 0);
        let client_recv = client.recv(&mut msg, 0);
        let server_send_client_recv = server_send.and_then(|_| client_recv);
        reactor.run(server_send_client_recv).unwrap();
    }

    println!("REP: {}", msg.as_str().unwrap());

    ::std::process::exit(0);
}
