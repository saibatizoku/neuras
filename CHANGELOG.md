## [0.1.2] - In development
### Added
- added basis for an actor-like framework
- using `uuid` crate for UUIDs.
- added ctrl-c interrupt handling capabilities
- `KeysCertificate` stores `z85encode`d `zmq::CurveKeyPair` in `TOML` files
- `Clock` to uniformly handle monotonic and system time
## [0.1.1] - 2017-11-29
### Added
- Support for ZMQ-tokio integration (experimental)
- Basic example for secure CurveKeyPair sockets
