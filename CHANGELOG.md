## [0.1.3] - 2020-03-07
### Added
- Travis CI with zmq support.
- Polling mechanism for multiple sockets using `mio`.

### Changed
- `mio` crate is no longer optional.
- Removed nightly feature and bare trait objects in examples/actorling.rs.
- rustfmt

## [0.1.2] - 2018-02-08
### Added
- added `RASPBIAN.md` for Raspbian-related documentation.
- added description of crate features to `README.md`.
- added "examples/tokio-req-rep.rs" to show using standard and tokio sockets in the same thread.
- added "Future", "Stream", and "Sink" types for sockets
- new Cargo feature: "async-tokio", for tokio-compatibility
- new Cargo feature: "async-mio", for mio-compatibility
- added basis for an actor-like framework
- using `uuid` crate for UUIDs.
- added ctrl-c interrupt handling capabilities
- `KeysCertificate` stores `z85encode`d `zmq::CurveKeyPair` in `TOML` files
- `Clock` to uniformly handle monotonic and system time

###[Changed]
- moved Raspbian-related documentation from `README.md` to `RASPBIAN.md`.

## [0.1.1] - 2017-11-29
### Added
- Support for ZMQ-tokio integration (experimental)
- Basic example for secure CurveKeyPair sockets
