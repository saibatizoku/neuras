extern crate neuras;
extern crate zmq;

use neuras::actor::Actorling;
use neuras::actor::errors::*;
use zmq::{Message, Sendable, Socket};

fn send_cmd<T>(pipe: &Socket, msg: T, response: &mut Message) -> Result<()>
where
    T: Sendable + ::std::fmt::Debug,
{
    println!("command: {:?}", msg);
    pipe.send(msg, 0).unwrap();
    pipe.recv(response, 0).unwrap();
    Ok(())
}

fn setup_actor_at(addr: &str) -> Actorling {
    let actorling = Actorling::new(addr).unwrap();
    // set a timeout for blocking operations (send/recv).
    // we use 500 ms. When sockets timeout, they return an error.
    actorling.pipe().set_rcvtimeo(500).unwrap();
    actorling.pipe().set_sndtimeo(500).unwrap();
    actorling
}

fn setup_actor() -> Actorling {
    setup_actor_at("inproc://test_actor")
}

#[test]
fn pipe_start_ping_and_stop() {
    let actorling = setup_actor();
    let pipe = actorling.pipe();
    let mut msg = Message::new();

    actorling.start().unwrap();

    {
        actorling.pipe().recv(&mut msg, 0).unwrap();
        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert_eq!("inproc://test_actor", status);
    }

    {
        send_cmd(pipe, "$PING", &mut msg).unwrap();

        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert_eq!("$PONG", status);
    }

    {
        send_cmd(pipe, "NON-exisiting-CMD", &mut msg).unwrap();

        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert_eq!("$WONTDO", status);
    }

    {
        send_cmd(pipe, "$STOP", &mut msg).expect("stop signal was not sent");

        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert_eq!("$STOPPING", status);
    }

    // trying to stop a stopped actor prints out to stderr that the
    // command was not confirmed, but returns ok.
    assert!(actorling.stop().is_ok());
}

#[test]
fn actor_uses_dynamic_sockets_on_tcp() {
    let actorling = setup_actor_at("tcp://127.0.10.1:*");
    let pipe = actorling.pipe();
    let mut msg = Message::new();

    actorling.start().unwrap();

    {
        actorling.pipe().recv(&mut msg, 0).unwrap();
        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert!(status.starts_with("tcp://127.0.10.1:"));
    }
}

#[test]
fn actor_can_create_other_actors() {
    let actorling = setup_actor_at("tcp://127.0.10.1:*");
    let pipe = actorling.pipe();
    let mut msg = Message::new();

    {
        actorling.start().unwrap();
        pipe.recv(&mut msg, 0).unwrap();
        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert!(status.starts_with("tcp://127.0.10.1:"));
    }
}
