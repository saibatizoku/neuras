neuras - A high-level API for networking with ØMQ ("zeromq") and tokio.
=======================================================================

## About

An attempt at having a [Rust](http://rust-lang.org) high-level API on top of [ØMQ](http://zeromq.org) (aka. _ZeroMQ_, _zmq_),
as suggested by
"[Features of a Higher-Level API](http://zguide.zeromq.org/page:all#toc74)",
by using tokio's reactor and tools.

This library uses [rust-zmq](https://github.com/erickt/rust-zmq)'s bindings under the hood.

## Justification

ZeroMQ provides a layer of abstraction over common patterns found in network programming. It has a very well-documented API that hides away the underlying complexity of how data is coded/decoded, transmitted, and made available. Its native support for asynchronous communication, peer-to-peer encrypted messaging, as well as the ever-increasing availability of socket options, make it simple and robust to work with.

[ØMQ - The Guide](http://zguide.zeromq.org/page:all) offers a collection of very well thought-out examples, use cases, and the author's clear thoughts regarding concepts such as code quality, design of network infrastructure. It is the first place to go when wanting to learn ØMQ.

## Installation

### Requirements

- [ØMQ](http://zeromq.org). Using version >= 4.0.0
- [rust-zmq](https://github.com/erickt/rust-zmq). Using master branch on git.









## Feature Wish List

- [ ] Automatic handling of sockets
- [ ] Portable thread management
- [ ] Piping from parent to child threads
- [X] Portable clocks
- [ ] A reactor to replace `zmq_poll()`
- [ ] Proper handling of `Ctrl-C`
