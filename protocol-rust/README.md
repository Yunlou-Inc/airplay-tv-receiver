# protocol-rust

A defensive Rust skeleton for receiver-side AirPlay protocol experimentation:
bounded RTSP parsing, device/discovery models, and an explicit session state machine.

This is **not** an implementation of Apple's private cryptography, authentication,
decryption, or full media compatibility. Advertised profile values must be selected
by the integrator and must not imply unsupported capabilities.

Run tests with `cargo test --workspace`. Install `cargo-fuzz` separately to run
`cargo fuzz run rtsp_request` from `fuzz/`.

## Control-plane boundary

`Router` is a pure in-memory request router: callers parse bytes with `parse_request`,
then route the resulting request and serialize the bounded `Response` themselves.
It performs no network I/O. It validates monotonic, unique `CSeq`, session ownership,
and lifecycle transitions. `/info` reports only configured profile fields, and TXT
records are generated from the same profile. Pairing/authentication cryptography and
media payload decryption are intentionally not implemented; applications must not
advertise those capabilities through `FeatureSet` until supplied by a separate,
audited layer.
