#[macro_use]
extern crate error_chain;
extern crate neuras;
extern crate zmq;

use zmq::{Context, CurveKeyPair, Message, Socket, SocketType};

error_chain! {
    errors {
        ReceiverBind {
            description ("could not bind receiver")
        }
        ReceiverDisconnect {
            description ("could not disconnect receiver")
        }
        SenderConnect {
            description ("could not connect sender")
        }
        SenderDisconnect {
            description ("could not disconnect sender")
        }
        SetupReceiver {
            description ("receiver was not setup")
        }
        SetupSender {
            description ("sender was not setup")
        }
    }
    links {
        Neuras(neuras::errors::Error, neuras::errors::ErrorKind);
    }
    foreign_links {
        Zmq(zmq::Error);
    }
}

struct SocketReceiver {
    endpoint: String,
    keys: CurveKeyPair,
    socket: Socket,
}

impl SocketReceiver {
    fn new(socket: Socket, endpoint: String, keys: CurveKeyPair) -> Result<SocketReceiver> {
        Ok(SocketReceiver {
            socket,
            endpoint,
            keys,
        })
    }

    fn bind(&self) -> Result<()> {
        self.socket.set_curve_server(true)?;
        self.socket.set_curve_publickey(&self.keys.public_key)?;
        self.socket.set_curve_secretkey(&self.keys.secret_key)?;
        self.socket
            .bind(&self.endpoint)
            .chain_err(|| ErrorKind::ReceiverBind)
    }

    fn disconnect(&self) -> Result<()> {
        println!("receiver disconnecting from: {:?}", &self.endpoint);
        self.socket
            .disconnect(&self.endpoint)
            .chain_err(|| ErrorKind::ReceiverDisconnect)
    }

    fn public_key(&self) -> &[u8] {
        self.keys.public_key.as_ref()
    }
}

struct SocketSender {
    endpoint: String,
    keys: CurveKeyPair,
    socket: Socket,
}

impl SocketSender {
    fn new(socket: Socket, endpoint: String, keys: CurveKeyPair) -> Result<SocketSender> {
        Ok(SocketSender {
            socket,
            endpoint,
            keys,
        })
    }

    fn connect(&self, server_key: &[u8]) -> Result<()> {
        self.socket.set_curve_serverkey(server_key)?;

        self.socket.set_curve_publickey(&self.keys.public_key)?;
        self.socket.set_curve_secretkey(&self.keys.secret_key)?;
        self.socket
            .connect(&self.endpoint)
            .chain_err(|| ErrorKind::SenderConnect)
    }

    fn disconnect(&self) -> Result<()> {
        println!("sender disconnecting from: {:?}", &self.endpoint);
        self.socket
            .disconnect(&self.endpoint)
            .chain_err(|| ErrorKind::SenderDisconnect)
    }
}

fn setup_sender(sock_type: SocketType, ctx: &Context, ep: &str) -> Result<SocketSender> {
    println!("Setting up sender type: {:?}", &sock_type);
    let socket = ctx.socket(sock_type)?;
    let keys = CurveKeyPair::new()?;
    let endpoint = ep.to_string();

    // sender socket, acts as client
    SocketSender::new(socket, endpoint, keys).chain_err(|| ErrorKind::SetupSender)
}

fn setup_receiver(sock_type: SocketType, ctx: &Context, ep: &str) -> Result<SocketReceiver> {
    println!("Setting up receiver type: {:?}", &sock_type);
    let receiver = ctx.socket(sock_type)?;
    let keys = CurveKeyPair::new()?;
    let endpoint = ep.to_string();

    // receiver socket acts as server, will accept connections
    SocketReceiver::new(receiver, endpoint, keys).chain_err(|| ErrorKind::SetupReceiver)
}

fn create_socketpair(
    endpoint: &str,
    context: Option<Context>,
) -> Result<(SocketSender, SocketReceiver)> {
    let ctx = match context {
        Some(c) => c,
        None => Context::new(),
    };

    let receiver = setup_receiver(zmq::PAIR, &ctx, endpoint)?;
    receiver.bind()?;

    let ep = receiver.socket.get_last_endpoint()?.unwrap();

    let sender = setup_sender(zmq::PAIR, &ctx, &ep)?;

    //receiver.bind("tcp://127.0.0.1:*").unwrap();
    println!("bound to {:?}", &ep);
    sender.connect(&receiver.public_key())?;
    Ok((sender, receiver))
}

fn run_code() -> Result<()> {
    let endpoint = "ipc://secure";

    let net_ctx = Context::new();

    let (sender, receiver) = create_socketpair(endpoint, Some(net_ctx))?;

    let mut msg = Message::new();

    sender.socket.send("foo".as_bytes(), 0)?;

    receiver.socket.recv(&mut msg, 0)?;

    assert_eq!(&msg[..], b"foo");
    assert_eq!(msg.as_str(), Some("foo"));
    println!("this is it {0}", msg.as_str().unwrap());
    assert_eq!(format!("{:?}", msg), "[102, 111, 111]");

    receiver.socket.send("baraboom".as_bytes(), 0)?;
    receiver.disconnect()?;

    sender.socket.recv(&mut msg, 0)?;
    sender.disconnect()?;

    assert_eq!(&msg[..], b"baraboom");
    println!("...{}", msg.as_str().unwrap());
    Ok(())
}

quick_main!(run_code);
