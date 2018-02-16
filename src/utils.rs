//! Functions for easy handling of ZMQ sockets, services, and enpoints.
use std::str::FromStr;

use url::Url;
use zmq;

use errors::*;

/// Create a ZMQ Context.
pub fn create_context() -> zmq::Context {
    zmq::Context::new()
}

/// Create a ZMQ Message.
pub fn create_message() -> Result<zmq::Message> {
    Ok(zmq::Message::new())
}

/// Subscribe a client to a PUB socket.
pub fn subscribe_client(subscriber: &zmq::Socket, channel: &str) -> Result<()> {
    subscriber
        .set_subscribe(channel.as_bytes())
        .chain_err(|| ErrorKind::Neurotic)
}

/// Proxy for XPUB/XSUB services.
pub fn zmq_xpub_xsub_proxy(context: &zmq::Context, xpub: &str, xsub: &str) -> Result<()> {
    let mut backend = zmq_xpub(context)?;
    let mut frontend = zmq_xsub(context)?;

    bind_socket(&backend, xsub)?;
    connect_socket(&frontend, xpub)?;

    zmq::proxy(&frontend, &backend).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as XPUB.
pub fn zmq_xpub(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::XPUB).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as XSUB.
pub fn zmq_xsub(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::XSUB).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as PUB.
pub fn zmq_pub(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::PUB).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as SUB.
pub fn zmq_sub(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::SUB).chain_err(|| ErrorKind::Neurotic)
}
/// Returns a ZMQ Socket configured as REP.
pub fn zmq_rep(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::REP).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as REQ.
pub fn zmq_req(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::REQ).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as PAIR.
pub fn zmq_pair(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::PAIR).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as PULL.
pub fn zmq_pull(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::PULL).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket configured as PUSH.
pub fn zmq_push(context: &zmq::Context) -> Result<zmq::Socket> {
    context.socket(zmq::PUSH).chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket bound to the specified address.
pub fn bind_socket(server: &zmq::Socket, addr: &str) -> Result<()> {
    let addr_url: Url = addr.parse().chain_err(|| ErrorKind::AddressParse)?;
    server
        .bind(addr_url.as_str())
        .chain_err(|| ErrorKind::Neurotic)
}

/// Returns a ZMQ Socket connected to the specified address.
pub fn connect_socket(client: &zmq::Socket, addr: &str) -> Result<()> {
    match Url::from_str(addr) {
        Ok(_) => client.connect(addr).chain_err(|| ErrorKind::Neurotic),
        Err(_) => Err(ErrorKind::AddressParse.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_request_client() {
        let ctx = zmq::Context::new();
        let req = zmq_req(&ctx).unwrap();
        assert_eq!(req.get_socket_type(), Ok(zmq::REQ));
    }

    // With these tests, we are making sure that we can properly parse
    // the urls that our crate uses
    #[test]
    fn decode_inproc_endpoint() {
        let socket_addr = "inproc:/tmp/hello";
        let parsed: Url = socket_addr.parse().unwrap();
        assert_eq!(parsed.scheme(), "inproc");
        assert_eq!(parsed.host_str(), None);
    }

    #[test]
    fn decode_ipc_endpoint() {
        let socket_addr = "ipc:/tmp/hello";
        let parsed: Url = socket_addr.parse().unwrap();
        assert_eq!(parsed.scheme(), "ipc");
        assert_eq!(parsed.host_str(), None);
    }

    #[test]
    fn decode_generic_tcp_endpoint() {
        let socket_addr = "tcp://*:5566";
        let parsed: Url = socket_addr.parse().unwrap();
        assert_eq!(parsed.scheme(), "tcp");
        assert_eq!(parsed.port(), Some(5566));
        assert_eq!(parsed.host_str(), Some("*"));
    }
}
