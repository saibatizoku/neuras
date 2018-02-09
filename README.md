neuras - A high-level API for networking with ØMQ in Rust
=========================================================

## About

An attempt at having a [Rust](http://rust-lang.org) high-level API on top of [ØMQ](http://zeromq.org) (aka. _ZeroMQ_, _zmq_),
as suggested by
"[Features of a Higher-Level API](http://zguide.zeromq.org/page:all#toc74)",
by using tokio's reactor and tools.

This library uses [rust-zmq](https://github.com/erickt/rust-zmq)'s bindings under the hood.

## Justification

ZeroMQ provides a layer of abstraction over common patterns found in network programming. It has a very well-documented API that hides away the underlying complexity of how data is coded/decoded, transmitted, and made available. Its native support for asynchronous communication, peer-to-peer encrypted messaging, as well as the ever-increasing availability of socket options, make it simple and robust to work with.

[ØMQ - The Guide](http://zguide.zeromq.org/page:all) offers a collection of very well thought-out examples, use cases, and the author's clear thoughts regarding concepts such as code quality, design of network infrastructure. It is the first place to go when wanting to learn ØMQ.

## Dependencies

- [ØMQ](http://zeromq.org). Using version >= 4.0.0
- [rust-zmq](https://github.com/erickt/rust-zmq). Using master branch on git.

## Installation

Add this to your `Cargo.toml`, under `[dependencies]`:

```
[dependencies]
neuras = { git = "https://github.com/saibatizoku/neuras" }
```

Then, to `src/lib.rs`, or `src/main.rs`:

```
extern crate neuras;

use neuras;
```

### Misc

Here is a quick guide to [setting up zeromq with Rasbpian](RASPBIAN.md).

## Feature Wish List

- [X] Automatic handling of sockets
      Rust handles dropping sockets once the context goes out of scope.
- [X] Portable thread management
      Rust handles portable thread management.
- [ ] Piping from parent to child threads
- [X] Portable clocks
- [X] A reactor to replace `zmq_poll()`
      Use `mio` for bare-metal I/O.
      Use `tokio-core` for fast, and reliable asynchronous I/O.
- [ ] Proper handling of `Ctrl-C`
