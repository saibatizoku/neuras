//! Streams for tokio-compatible sockets.
use super::super::SocketRecv;

use std::io;

use futures::{Async, Poll, Stream};
use zmq;

/// Single-message stream for sockets.
pub struct MessageStream<'a, T: 'a> {
    socket: &'a T,
}

impl<'a, T> MessageStream<'a, T>
where
    T: SocketRecv + 'a,
{
    pub fn new(socket: &'a T) -> MessageStream<'a, T> {
        MessageStream { socket }
    }
}

impl<'a, T> Stream for MessageStream<'a, T>
where
    T: SocketRecv + 'a,
{
    type Item = zmq::Message;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut buf = zmq::Message::new();
        match SocketRecv::recv(self.socket, &mut buf, 0) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(_) => Ok(Async::Ready(Some(buf))),
        }
    }
}

/// Multipart-message stream for sockets.
pub struct MessageMultipartStream<'a, T: 'a> {
    socket: &'a T,
}

impl<'a, T> MessageMultipartStream<'a, T>
where
    T: SocketRecv + 'a,
{
    pub fn new(socket: &'a T) -> MessageMultipartStream<'a, T> {
        MessageMultipartStream { socket }
    }
}

impl<'a, T> Stream for MessageMultipartStream<'a, T>
where
    T: SocketRecv + 'a,
{
    type Item = Vec<zmq::Message>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match SocketRecv::recv_multipart(self.socket, 0) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(vecs) => {
                let msgs = vecs.iter().map(|v| {
                    v.into()
                }).collect();
                Ok(Async::Ready(Some(msgs)))
            }
        }
    }
}

