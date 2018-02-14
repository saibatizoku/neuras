extern crate neuras;
extern crate zmq;

use neuras::actor::Actorling;
use neuras::actor::errors::*;

fn send_cmd<T: zmq::Sendable + ::std::fmt::Debug>(pipe: &zmq::Socket, msg: T, response: &mut zmq::Message) -> Result<()> {
    println!("command: {:?}", msg);
    let _ = pipe.send(msg, 0).unwrap();
    let _ = pipe.recv(response, 0).unwrap();
    Ok(())
}

#[test]
fn main () {
    let actorling = Actorling::new("inproc://test_actor").unwrap();
    let _ = actorling.start().unwrap();
    let mut msg = zmq::Message::new();
    let pipe = actorling.pipe();

    {
        let _ = send_cmd(pipe, "PING", &mut msg).unwrap();

        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert_eq!("PONG", status);
    }

    {
        let _ = send_cmd(pipe, "NON-exisiting-CMD", &mut msg).unwrap();

        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert_eq!("ERR", status);
    }

    {
        let _ = send_cmd(pipe, "$STOP", &mut msg).unwrap();

        let status = msg.as_str().unwrap();
        println!("response: {}", &status);
        assert_eq!("OK", status);
    }

    // assert that trying to stop an already stopped actorling is an error.
    assert!(actorling.stop().is_err());

    ::std::process::exit(0);
}
