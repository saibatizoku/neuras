neuras - A high-level API for networking with ØMQ in Rust
=========================================================

[![Build Status](https://travis-ci.org/saibatizoku/neuras.svg?branch=master)](https://travis-ci.org/saibatizoku/neuras)

## About

An attempt at having a [Rust](http://rust-lang.org) high-level API on top of [ØMQ](http://zeromq.org) (aka. _ZeroMQ_, _zmq_),
as suggested by
"[Features of a Higher-Level API](https://zguide.zeromq.org/docs/chapter3/#Features-of-a-Higher-Level-API)",
by using tokio's reactor and tools.

This library uses [rust-zmq](https://github.com/erickt/rust-zmq)'s bindings under the hood.

## Justification

ZeroMQ provides a layer of abstraction over common patterns found in network programming. It has a very well-documented API that hides away the underlying complexity of how data is coded/decoded, transmitted, and made available. Its native support for asynchronous communication, peer-to-peer encrypted messaging, as well as the ever-increasing availability of socket options, make it simple and robust to work with.

[ØMQ - The Guide](http://zguide.zeromq.org/page:all) offers a collection of very well thought-out examples, use cases, and the author's clear thoughts regarding concepts such as code quality, design of network infrastructure. It is the first place to go when wanting to learn ØMQ.

## Dependencies

- [ØMQ](http://zeromq.org). Using version >= 4.0.0
- [rust-zmq](https://github.com/erickt/rust-zmq). Using version "0.9".

## Installation

### Update `Cargo.toml`

Add this to your `Cargo.toml`, under `[dependencies]`:

**`default`**
The `default` feature uses the standard `zmq::Socket`. Use this if you have existing `zmq` code, or if you want to use sockets in blocking mode.

Also, the optional feature `async-tokio` is a default feature.

```
[dependencies]
neuras = { git = "https://github.com/saibatizoku/neuras" }
```

#### Optional features

**`async-tokio`**

The `async-tokio` feature uses `neuras::socket::tokio::TokioSocket`, leveraging `mio`, which has methods for messaging asynchronously with the `Future`, `Stream`, and `Sink` traits. Use this if you want to use sockets with `tokio_core::reactor::Core`. Socket messaging is non-blocking.

```
[dependencies.neuras]
git = "https://github.com/saibatizoku/neuras"
features = ["async-tokio"]
```

### Use in `src/lib.rs`, or `src/main.rs`:

```
extern crate neuras;

use neuras;
```

## Examples

Please see [examples/tokio-req-rep.rs](examples/tokio-req-rep.rs) for a working example of how to use the REQ-REP messaging pattern with a tokio reactor.

More examples soon!

## Feature Wish List and Work-In-Progress

- [X] *Automatic handling of sockets* Rust handles memory-safety and automatically enforces that lifetimes stay within the program scope. What this means for us, is that sockets will be dropped when their context goes out of scope.
- [X] *Portable thread management* Rust handles concurrency using OS-portable threads.
- [X] *Portable clocks* Rust offers OS-portable clocks in the standard library module `std::time`.
- [X] *A reactor to replace `zmq_poll()`* 
- [ ] Piping from parent to child threads
- [ ] Proper handling of `Ctrl-C`

## Misc

Here is a quick guide to [setting up zeromq with Rasbpian](RASPBIAN.md).
