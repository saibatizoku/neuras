//! Polling for evented actor types.
use super::socket::{PollingSocket, SocketRecv};

use failure::Error;
use mio_lib::event::Evented;
use mio_lib::{Events, Poll, Token};
use slab::Slab;
use std::io;
use std::time::Duration;
use zmq;

/// Polling instance for evented actors.
pub struct Poller {
    context: zmq::Context,
    pub poll: Poll,
    pub actors: Slab<Box<Evented>>,
}

impl Poller {
    /// create a new `Poller` instance. Has a default capacity for 10 actors, which is useful
    /// for non-production usage. You should probably use `Poller::with_capacity` to meet your
    /// specific needs. Basically, if the capacity is reserved at compile-time, you get
    /// pre-allocated and cheap insert/remove/get/get_mut operations for actors, which means that
    /// you can avoid memory fragmentationa with no additional effort. Plan ahead, it pays off!
    pub fn new() -> Poller {
        Poller::with_context_and_capacity(zmq::Context::new(), 0)
    }

    /// create a new `Poller` instance with a pre-allocation for pollable items.
    pub fn with_capacity(capacity: usize) -> Poller {
        Poller::with_context_and_capacity(zmq::Context::new(), capacity)
    }

    /// create a new `Poller` instance with an existing context, and no pre-allocation.
    pub fn with_context(context: zmq::Context) -> Poller {
        Poller::with_context_and_capacity(context, 0)
    }

    /// create a new `Poller` instance with an existing context, and pre-allocation
    /// for pollable items.
    pub fn with_context_and_capacity(context: zmq::Context, capacity: usize) -> Poller {
        let poll = Poll::new().expect("mio poll could not be created");
        let actors = Slab::with_capacity(capacity);
        Poller {
            context,
            poll,
            actors,
        }
    }
}

impl Default for Poller {
    fn default() -> Self {
        Poller::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zmq;

    #[test]
    fn default_poller_has_no_pre_allocation() {
        let poller: Poller = Default::default();
        assert_eq!(poller.actors.capacity(), 0);
    }

    #[test]
    fn new_poller_can_be_created_with_capacity() {
        let poller: Poller = Poller::with_capacity(20);
        assert_eq!(poller.actors.capacity(), 20);
    }

    #[test]
    fn new_poller_can_be_created_with_existing_context_and_capacity() {
        let context = zmq::Context::new();
        let ctx = context.clone();
        let poller: Poller = Poller::with_context_and_capacity(ctx, 30);
        assert_eq!(poller.actors.capacity(), 30);
    }
}
