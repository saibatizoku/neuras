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


### Installing ZMQ on Raspbian

`neuras` is meant to be used on any linux box, including Raspberry Pi's, and similar. These steps can be seen as general guidelines to getting ZMQ on your box, consult your package-management, who knows, maybe you already have the latest stable version running. If so, you can avoid manual installation.

1.  Remove any ZMQ-related packages from Raspbian (libzmq/libzmq3 y libzmq-dev/libzmq3-dev)
    ```
    sudo apt-get remove libzmq libzmq-dev
    ```
    or
    ```
    sudo apt-get remove libzmq3 libzmq3-dev
    ```
2.  Install libsodium/libsodium-dev & libunwind/libunwind-dev

    `sudo apt-get install libsodium libsodium-dev libunwind libunwind-dev`

3.  Download [zeromq-4.2.1](https://github.com/zeromq/libzmq/releases/download/v4.2.1/zeromq-4.2.1.tar.gz), or latest.
4.  Unpack, configure, make and install

    ```
    ./configure --with-libsodium --libdir=/usr/lib/arm-linux-gnueabihf --includedir=/usr/include
    make
    sudo make install
    ```

    For other configuration options, `./configure --help` is the way to go.


## Feature Wish List

- [ ] Automatic handling of sockets
- [ ] Portable thread management
- [ ] Piping from parent to child threads
- [X] Portable clocks
- [ ] A reactor to replace `zmq_poll()`
- [ ] Proper handling of `Ctrl-C`
