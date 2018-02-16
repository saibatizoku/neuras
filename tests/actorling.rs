extern crate neuras;
extern crate zmq;

use neuras::actor::Actorling;
use neuras::actor::errors::*;

fn send_cmd<T>(pipe: &zmq::Socket, msg: T, response: &mut zmq::Message) -> Result<()>
where
    T: zmq::Sendable + ::std::fmt::Debug,
{
    println!("command: {:?}", msg);
    let _ = pipe.send(msg, 0).unwrap();
    let _ = pipe.recv(response, 0).unwrap();
    Ok(())
}

#[test]
fn main() {
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

    // trying to stop a stopped actor prints out to stderr that the
    // command was not confirmed, but returns ok.
    assert!(actorling.stop().is_ok());

    ::std::process::exit(0);
}
