//! Sinks for tokio-compatible sockets.
use super::super::SocketSend;

use std::io;
use std::ops::Deref;

use futures::{Async, AsyncSink, Poll, Sink, StartSend};
use zmq;

/// Single-message sink for sockets.
pub struct MessageSink<'a, T: 'a> {
    socket: &'a T,
}

impl<'a, T> MessageSink<'a, T>
where
    T: SocketSend + 'a,
{
    pub fn new(socket: &'a T) -> MessageSink<'a, T> {
        MessageSink { socket }
    }
}

impl<'a, T> Sink for MessageSink<'a, T>
where
    T: SocketSend + 'a,
{
    type SinkItem = zmq::Message;
    type SinkError = io::Error;

    fn start_send(&mut self, item: zmq::Message) -> StartSend<zmq::Message, Self::SinkError> {
        match SocketSend::send(self.socket, item.deref(), 0) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(AsyncSink::NotReady(item))
                } else {
                    Err(e)
                }
            }
            Ok(_) => {
                Ok(AsyncSink::Ready)
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}

/// Multipart-message sink for sockets.
pub struct MessageMultipartSink<'a, T: 'a> {
    socket: &'a T,
}

impl<'a, T> MessageMultipartSink<'a, T>
where
    T: SocketSend + 'a,
{
    pub fn new(socket: &'a T) -> MessageMultipartSink<'a, T> {
        MessageMultipartSink { socket }
    }
}

impl<'a, T> Sink for MessageMultipartSink<'a, T>
where
    T: SocketSend + 'a,
{
    type SinkItem = Vec<Vec<u8>>;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Vec<Vec<u8>>) -> StartSend<Vec<Vec<u8>>, Self::SinkError> {
        match SocketSend::send_multipart(self.socket, &item, 0) {
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    Ok(AsyncSink::NotReady(item))
                } else {
                    Err(e)
                }
            }
            Ok(_) => {
                Ok(AsyncSink::Ready)
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}
