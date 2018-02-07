//! Futures for tokio-compatible sockets.
use super::super::{SocketRecv, SocketSend};
use super::TokioSocket;

use std::io;
use std::ops::Deref;
use futures::{Async, Future, Poll};
use zmq::Message;

/// A Future that sends a `Message`.
pub struct SendMessage<'a> {
    socket: &'a TokioSocket<'a>,
    message: Message,
    flags: i32,
}

impl<'a> SendMessage<'a> {
    /// Create a new `SendMessage` future.
    pub fn new<M: Into<Message>>(socket: &'a TokioSocket, msg: M, flags: i32) -> SendMessage<'a> {
        let message = msg.into();
        SendMessage {
            socket,
            message,
            flags,
        }
    }
}

impl<'a> Future for SendMessage<'a> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketSend::send(self.socket, self.message.deref(), self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(_) => Ok(Async::Ready(())),
        }
    }
}

/// A Future that sends a multi-part `Message`.
pub struct SendMultipartMessage<'a> {
    socket: &'a TokioSocket<'a>,
    messages: Vec<Vec<u8>>,
    flags: i32,
}

impl<'a> SendMultipartMessage<'a> {
    /// Create a new `SendMultipartMessage`.
    pub fn new<I, M>(socket: &'a TokioSocket, iter: I, flags: i32) -> SendMultipartMessage<'a>
    where
        I: IntoIterator<Item = M>,
        M: Into<Vec<u8>>,
    {
        let messages: Vec<Vec<u8>> = iter.into_iter().map(|m| m.into()).collect();
        SendMultipartMessage {
            socket,
            messages,
            flags,
        }
    }
}

impl<'a> Future for SendMultipartMessage<'a> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketSend::send_multipart(self.socket, &self.messages, self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(_) => Ok(Async::Ready(())),
        }
    }
}

/// A Future that receives a `Message` asynchronously.
pub struct RecvMessage<'a, 'b> {
    socket: &'a TokioSocket<'a>,
    msg: &'b mut Message,
    flags: i32,
}

impl<'a, 'b> RecvMessage<'a, 'b> {
    pub fn new(socket: &'a TokioSocket, msg: &'b mut Message, flags: i32) -> RecvMessage<'a, 'b> {
        RecvMessage { socket, msg, flags }
    }
}

impl<'a, 'b> Future for RecvMessage<'a, 'b> {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketRecv::recv(self.socket, self.msg, self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(_) => Ok(Async::Ready(())),
        }
    }
}

/// A Future that receives a multi-part `Message` asynchronously.
pub struct RecvMultipartMessage<'a> {
    socket: &'a TokioSocket<'a>,
    flags: i32,
}

impl<'a> RecvMultipartMessage<'a> {
    pub fn new(socket: &'a TokioSocket, flags: i32) -> RecvMultipartMessage<'a> {
        RecvMultipartMessage { socket, flags }
    }
}

impl<'a> Future for RecvMultipartMessage<'a> {
    type Item = Vec<Message>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match SocketRecv::recv_multipart(self.socket, self.flags) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(Async::NotReady)
                } else {
                    Err(e)
                }
            }
            Ok(msgs) => {
                let m_out = msgs.iter().map(|v| v.into()).collect::<Vec<Message>>();
                Ok(Async::Ready(m_out))
            }
        }
    }
}

/// API for socket futures with tokio.
///
/// This trait is meant for types that will execute in asynchronous tasks.
pub trait SocketFutures<'a> {
    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    fn async_send<M: Into<Message>>(&self, message: M, flags: i32) -> SendMessage;

    /// Sends a type implementing `Into<zmq::Message>` as a `Future`.
    fn async_send_multipart<I, M>(&self, messages: I, flags: i32) -> SendMultipartMessage
    where
        I: IntoIterator<Item = M>,
        M: Into<Vec<u8>>;

    /// Returns a `Future` that resolves into a `zmq::Message`
    fn async_recv<'b>(&'a self, msg: &'b mut Message, flags: i32) -> RecvMessage<'a, 'b>;

    /// Returns a `Future` that resolves into a `Vec<zmq::Message>`
    fn async_recv_multipart(&self, flags: i32) -> RecvMultipartMessage;
}
