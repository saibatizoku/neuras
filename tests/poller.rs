extern crate mio;
extern crate neuras;
extern crate zmq;

use std::io;
use mio::{Evented, Poll, PollOpt, Ready, Token};
use neuras::socket::PollingSocket;
use neuras::poller::Poller;

struct ResponderActor {
    inner: PollingSocket,
}

impl Evented for ResponderActor {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        Evented::register(&self.inner, poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        Evented::reregister(&self.inner, poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        Evented::deregister(&self.inner, poll)
    }
}

#[test]
fn run_poll() {
    assert!(true);
}
