//! Network sockets and tools for authentication, and encryption protocol for ZeroMQ.
//!
//! The underlying code uses an implementation of
//! [ZMTP-CURVE](https://rfc.zeromq.org/spec:25/ZMTP-CURVE),
//! by way of the [rust-zmq](https://github.com/erickt/rust-zmq) crate.
//!
pub mod errors {
    //! Errors for secure-socket communications.
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

use zmq::{Context, CurveKeyPair, Message, Sendable, Socket, SocketType};
use zmq::{z85_decode, z85_encode};

use super::initialize::sys_context;

use self::errors::*;

/// Certificates that can encode `zmq::CurveKeyPair` into `TOML` format.
/// Useful for authentication purposes.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct KeysCertificate {
    pub secret_key: String,
    pub public_key: String,
}

impl From<CurveKeyPair> for KeysCertificate {
    fn from(keys: CurveKeyPair) -> Self {
        KeysCertificate {
            secret_key: z85_encode(&keys.secret_key).unwrap(),
            public_key: z85_encode(&keys.public_key).unwrap(),
        }
    }
}

impl Into<CurveKeyPair> for KeysCertificate {
    fn into(self) -> CurveKeyPair {
        let secret_key = z85_decode(&self.secret_key).unwrap();
        let public_key = z85_decode(&self.public_key).unwrap();
        let mut keys = CurveKeyPair::new().unwrap();
        let mut key = [0u8; 32];
        let _ = key.copy_from_slice(&secret_key);
        keys.secret_key = key;
        let _ = key.copy_from_slice(&public_key);
        keys.public_key = key;
        keys
    }
}

/// Secures a ZMQ socket as a server, according to the
/// [ZMTP-CURVE](https://rfc.zeromq.org/spec:25/ZMTP-CURVE) specification.
pub fn secure_server_socket(socket: &Socket, keys: &CurveKeyPair) -> Result<()> {
    socket.set_curve_server(true)?;
    socket.set_curve_publickey(&keys.public_key)?;
    socket.set_curve_secretkey(&keys.secret_key)?;
    Ok(())
}

/// Configures a socket that connects securely to a ZMQ server, according to the
/// [ZMTP-CURVE](https://rfc.zeromq.org/spec:25/ZMTP-CURVE) specification.
pub fn secure_client_socket(socket: &Socket, server_key: &[u8], keys: &CurveKeyPair) -> Result<()> {
    socket.set_curve_serverkey(server_key)?;
    socket.set_curve_publickey(&keys.public_key)?;
    socket.set_curve_secretkey(&keys.secret_key)?;
    Ok(())
}

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
        let _cipher = secure_server_socket(&self.socket, &self.keys)?;
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
    /// the `server_key`, which is the public server key, and setting
    /// `public_key`/`secret_key` from `self.keys`.
    pub fn connect(&self, server_key: &[u8]) -> Result<()> {
        let _cipher = secure_client_socket(&self.socket, server_key, &self.keys)?;
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
    pub fn new() -> Result<CipherSocketBuilder> {
        let context = sys_context();
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

#[cfg(test)]
mod tests {
    use super::*;
    use toml;
    use zmq;

    const TEST_CERT_FILE: &str = r#"secret_key = "F05VvicyHYA1n0=5$7r5Yn(*(53Dgeb#@A7{SVy="
public_key = "!6X7EbrYp&d7?U.0&yrcL]lVlR:*eMaEh7R0Q!bh"
"#;
    const TEST_KEY_PAIR: zmq::CurveKeyPair = zmq::CurveKeyPair {
        secret_key: SECRET_KEY,
        public_key: PUBLIC_KEY,
    };
    const SECRET_KEY: [u8; 32] = [
        127, 145, 224, 130, 56, 117, 149, 163, 112, 14, 145, 50, 18, 153, 44, 215, 187, 142, 253,
        3, 15, 175, 6, 73, 37, 85, 211, 99, 247, 205, 76, 218,
    ];
    const PUBLIC_KEY: [u8; 32] = [
        211, 209, 252, 48, 35, 61, 92, 139, 40, 188, 60, 29, 2, 166, 123, 61, 149, 25, 172, 100,
        167, 70, 240, 81, 32, 149, 230, 67, 1, 238, 203, 0,
    ];
    const SECRET_HEX: &str = "7f91e082387595a3700e913212992cd7bb8efd030faf06492555d363f7cd4cda";
    const PUBLIC_HEX: &str = "d3d1fc30233d5c8b28bc3c1d02a67b3d9519ac64a746f0512095e64301eecb00";
    const SECRET_Z85: &str = "F05VvicyHYA1n0=5$7r5Yn(*(53Dgeb#@A7{SVy=";
    const PUBLIC_Z85: &str = "!6X7EbrYp&d7?U.0&yrcL]lVlR:*eMaEh7R0Q!bh";

    // Inefficient but terse base16 encoder
    fn print_as_hex(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<_>>()
            .join("")
    }

    fn setup_socket_n_keys(
        t: zmq::SocketType,
        c: Option<zmq::Context>,
    ) -> (zmq::Socket, zmq::CurveKeyPair) {
        let ctx = match c {
            Some(ct) => ct,
            None => zmq::Context::new(),
        };
        (ctx.socket(t).unwrap(), zmq::CurveKeyPair::new().unwrap())
    }

    #[test]
    fn secure_socket_as_a_server() {
        let (server, keys) = setup_socket_n_keys(zmq::PAIR, None);
        // test function
        let _secure = secure_server_socket(&server, &keys);

        // test that zmq socket configuration is as expected
        assert_eq!(server.is_curve_server().unwrap(), true);
        assert_eq!(
            &keys.secret_key,
            server.get_curve_secretkey().unwrap().as_slice()
        );
        assert_eq!(
            &keys.public_key,
            server.get_curve_publickey().unwrap().as_slice()
        );
    }

    #[test]
    fn secure_socket_as_a_client() {
        let (client, keys) = setup_socket_n_keys(zmq::PAIR, None);
        let server_key = zmq::CurveKeyPair::new().unwrap().public_key;
        // test function
        let _secure = secure_client_socket(&client, &server_key, &keys);

        // test that zmq socket configuration is as expected
        assert_eq!(client.is_curve_server().unwrap(), false);
        assert_eq!(
            &keys.secret_key,
            client.get_curve_secretkey().unwrap().as_slice()
        );
        assert_eq!(
            &keys.public_key,
            client.get_curve_publickey().unwrap().as_slice()
        );
        assert_eq!(
            &server_key,
            client.get_curve_serverkey().unwrap().as_slice()
        );
    }

    #[test]
    fn from_curve_keypair_to_keys_certificate_type() {
        let keys = TEST_KEY_PAIR;

        assert_eq!(SECRET_HEX, &print_as_hex(&keys.secret_key));
        assert_eq!(PUBLIC_HEX, &print_as_hex(&keys.public_key));

        let certificate = KeysCertificate::from(keys);

        assert_eq!(SECRET_Z85, &certificate.secret_key);
        assert_eq!(PUBLIC_Z85, &certificate.public_key);

        let secret = z85_decode(&certificate.secret_key).unwrap();
        assert_eq!(&print_as_hex(&secret), SECRET_HEX);

        let public = z85_decode(&certificate.public_key).unwrap();
        assert_eq!(&print_as_hex(&public), PUBLIC_HEX);
    }

    #[test]
    fn from_keys_certificate_to_curve_keypair_type() {
        let certificate = KeysCertificate {
            secret_key: SECRET_Z85.to_string(),
            public_key: PUBLIC_Z85.to_string(),
        };
        let keys: CurveKeyPair = certificate.into();
        assert_eq!(keys.secret_key, TEST_KEY_PAIR.secret_key);
        assert_eq!(keys.public_key, TEST_KEY_PAIR.public_key);
    }

    #[test]
    fn from_keys_certificate_to_toml_file() {
        let certificate = KeysCertificate {
            secret_key: SECRET_Z85.to_string(),
            public_key: PUBLIC_Z85.to_string(),
        };
        let toml = toml::to_string(&certificate).unwrap();

        assert_eq!(toml, TEST_CERT_FILE);
    }

    #[test]
    fn from_toml_to_keys_certificate_file() {
        let certificate: KeysCertificate = toml::from_str(TEST_CERT_FILE).unwrap();
        let keys: CurveKeyPair = certificate.into();
        assert_eq!(keys.secret_key, TEST_KEY_PAIR.secret_key);
        assert_eq!(keys.public_key, TEST_KEY_PAIR.public_key);
    }
}
