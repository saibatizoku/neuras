Neuras
======

Utilería de software, escrita en Rust, para la gestión de telemetría. Utiliza [rust-zmq](https://github.com/erickt/rust-zmq), que ofrece `bindings` para [ØMQ](http://zeromq.org/).

Neuras funciona con [benita](https://github.com/saibatizoku/benita).

## Compilando zeromq-4.2.1.tar.gz con cifrado en Raspberry Pi 3

1.  Desintalar cualquier instalación de la paquetería de Raspbian (libzmq/libzmq3 y libzmq-dev/libzmq3-dev)
    ```
    sudo apt-get remove libzmq libzmq-dev
    ```
    ó
    ```
    sudo apt-get remove libzmq3 libzmq3-dev
    ```
2.  Instalar libsodium/libsodium-dev y libunwind/libunwind-dev
3.  Descargar [zeromq-4.2.1](https://github.com/zeromq/libzmq/releases/download/v4.2.1/zeromq-4.2.1.tar.gz) o superior
4.  Compilar con los siguientes parámetros

    ```
    ./configure --with-libsodium --libdir=/usr/lib/arm-linux-gnueabihf --includedir=/usr/include
    make
    sudo make install
    ```

    En caso de necesitar especificar algún otro directorio, `./configure --help` tiene el listado completo de posibles directorios configurables.
