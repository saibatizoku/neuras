use zmq::{Context, CurveKeyPair, Message, Sendable, Socket, SocketType};

pub mod errors {
    use zmq;

    error_chain! {
        errors {
            LastBoundEndpoint {
                description ("failed obtaining last endpoint bound to this socket")
            }
            EndpointString {
                description ("failed to parse endpoint string")
            }
            ReceiverBind {
                description ("could not bind receiver")
            }
            ReceiverDisconnect {
                description ("could not disconnect receiver")
            }
            ReceiverSend {
                description ("receiver could not send message")
            }
            ReceiverReceive {
                description ("receiver could not receive message")
            }
            SenderConnect {
                description ("could not connect sender")
            }
            SenderDisconnect {
                description ("could not disconnect sender")
            }
            SenderSend {
                description ("sender could not send message")
            }
            SenderReceive {
                description ("sender could not receive message")
            }
            SetupReceiver {
                description ("receiver was not setup")
            }
            SetupSender {
                description ("sender was not setup")
            }
        }
        foreign_links {
            Zmq(zmq::Error);
        }
    }
}

use self::errors::*;

/// A socket that receives incoming messages from a ciphered connection.
pub struct CipherReceiver {
    endpoint: String,
    keys: CurveKeyPair,
    socket: Socket,
}

impl CipherReceiver {
    /// Create a new `CipherReceiver` from a given `zmq::Socket`,
    /// a given url `&str`, and the required `CurveKeyPair` for
    /// ciphered-communications.
    pub fn new(socket: Socket, url: &str, keys: CurveKeyPair) -> Result<CipherReceiver> {
        let endpoint = url.to_string();
        Ok(CipherReceiver {
            socket,
            endpoint,
            keys,
        })
    }

    /// Bind the receiver to `self.endpoint`, configuring the socket with
    /// `set_curve_server(true)`, and setting `public_key`/`secret_key`
    /// from `self.keys`.
    pub fn bind(&self) -> Result<()> {
        self.socket.set_curve_server(true)?;
        self.socket.set_curve_publickey(&self.keys.public_key)?;
        self.socket.set_curve_secretkey(&self.keys.secret_key)?;
        self.socket
            .bind(&self.endpoint)
            .chain_err(|| ErrorKind::ReceiverBind)
    }

    /// Calls the socket's disconnect method on `self.endpoint`, effectively
    /// unbinding the server.
    pub fn disconnect(&self) -> Result<()> {
        println!("receiver disconnecting from: {:?}", &self.endpoint);
        self.socket
            .disconnect(&self.endpoint)
            .chain_err(|| ErrorKind::ReceiverDisconnect)
    }

    /// Return the address of the last endpoint this socket was bound to.
    pub fn get_last_endpoint(&self) -> Result<String> {
        match self.socket
            .get_last_endpoint()
            .chain_err(|| ErrorKind::LastBoundEndpoint)?
        {
            Ok(ep) => Ok(ep),
            Err(_) => bail!(ErrorKind::EndpointString),
        }
    }

    /// Receive a message into a `Message`.
    pub fn recv(&self, msg: &mut Message, flags: i32) -> Result<()> {
        self.socket
            .recv(msg, flags)
            .chain_err(|| ErrorKind::ReceiverReceive)
    }

    /// Send a message.
    ///
    /// Due to the provided `From` implementations, this works for
    /// `&[u8]`, `Vec<u8>` and `&str` `Message` itself.
    pub fn send<T>(&self, data: T, flags: i32) -> Result<()>
    where
        T: Sendable,
    {
        self.socket
            .send(data, flags)
            .chain_err(|| ErrorKind::ReceiverSend)
    }

    /// Convenience method for accessing the socket's public key. It is needed
    /// for clients to connect to the `CipherReceiver`.
    pub fn public_key(&self) -> &[u8] {
        self.keys.public_key.as_ref()
    }
}

/// A socket that sends outgoing messages through a ciphered connection.
pub struct CipherSender {
    endpoint: String,
    keys: CurveKeyPair,
    socket: Socket,
}

impl CipherSender {
    /// Create a new `CipherSender` from a given `zmq::Socket`,
    /// a given url `&str`, and the required `CurveKeyPair` for
    /// ciphered-communications.
    pub fn new(socket: Socket, url: &str, keys: CurveKeyPair) -> Result<CipherSender> {
        let endpoint = url.to_string();
        Ok(CipherSender {
            socket,
            endpoint,
            keys,
        })
    }

    /// Connect the sender to `self.endpoint`, configuring the socket with
    /// the server public_key, and setting `public_key`/`secret_key`
    /// from `self.keys`.
    pub fn connect(&self, server_key: &[u8]) -> Result<()> {
        self.socket.set_curve_serverkey(server_key)?;

        self.socket.set_curve_publickey(&self.keys.public_key)?;
        self.socket.set_curve_secretkey(&self.keys.secret_key)?;
        self.socket
            .connect(&self.endpoint)
            .chain_err(|| ErrorKind::SenderConnect)
    }

    /// Calls the socket's disconnect method on `self.endpoint`, effectively
    /// disconnecting the client.
    pub fn disconnect(&self) -> Result<()> {
        println!("sender disconnecting from: {:?}", &self.endpoint);
        self.socket
            .disconnect(&self.endpoint)
            .chain_err(|| ErrorKind::SenderDisconnect)
    }

    /// Receive a message into a `Message`.
    pub fn recv(&self, msg: &mut Message, flags: i32) -> Result<()> {
        self.socket
            .recv(msg, flags)
            .chain_err(|| ErrorKind::SenderReceive)
    }

    /// Send a message.
    ///
    /// Due to the provided `From` implementations, this works for
    /// `&[u8]`, `Vec<u8>` and `&str` `Message` itself.
    pub fn send<T>(&self, data: T, flags: i32) -> Result<()>
    where
        T: Sendable,
    {
        self.socket
            .send(data, flags)
            .chain_err(|| ErrorKind::SenderSend)
    }
}

/// A type for building ciphered-sockets with a shared `zmq::Context`.
pub struct CipherSocketBuilder {
    context: Context,
}

impl CipherSocketBuilder {
    /// Create a new instance of `CipherSocketBuilder` with an optional
    /// `zmq::Context`. If no context is specified, a new one is created.
    pub fn new(ctx: Option<Context>) -> Result<CipherSocketBuilder> {
        let context = match ctx {
            Some(c) => c,
            None => Context::new(),
        };
        Ok(CipherSocketBuilder { context })
    }

    /// Create a new instance `CipherSender` that can be of `zmq::SocketType`
    /// and connected to the `&str` endpoint.
    pub fn sender(&self, socket_type: SocketType, endpoint: &str) -> Result<CipherSender> {
        println!("Setting up sender type: {:?}", &socket_type);
        let socket = self.context.socket(socket_type)?;
        let keys = CurveKeyPair::new()?;
        // sender socket, acts as client
        CipherSender::new(socket, endpoint, keys).chain_err(|| ErrorKind::SetupSender)
    }

    /// Create a new instance `CipherReceiver` that can be of `zmq::SocketType`
    /// and bound to the `&str` endpoint.
    pub fn receiver(&self, socket_type: SocketType, endpoint: &str) -> Result<CipherReceiver> {
        println!("Setting up receiver type: {:?}", &socket_type);
        let receiver = self.context.socket(socket_type)?;
        let keys = CurveKeyPair::new()?;
        // receiver socket acts as server, will accept connections
        CipherReceiver::new(receiver, endpoint, keys).chain_err(|| ErrorKind::SetupReceiver)
    }
}
