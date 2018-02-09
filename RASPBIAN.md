# Installing ZMQ on Raspbian

`neuras` is meant to be used on any linux box, including Raspberry Pi, and similar. These steps can be seen as general guidelines to getting ZMQ on your box, consult your package-management, who knows, maybe you already have the latest stable version running. If so, you can avoid manual installation.

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

