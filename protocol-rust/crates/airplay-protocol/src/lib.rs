//! Defensive protocol primitives for an AirPlay receiver.
//!
//! This crate deliberately does **not** implement private AirPlay cryptography,
//! authentication, decryption, or a complete media pipeline.

pub mod control;
pub mod discovery;
pub mod ffi;
pub mod frontend;
pub mod profile;
pub mod pairing;
pub mod rtsp;
pub mod session;

pub use control::{Response, RouteError, RouteLimits, Router};
pub use discovery::{DiscoveryError, DiscoveryRecord, ServiceKind, TxtRecord};
pub use profile::{DeviceProfile, FeatureSet, Model, ProtocolVersion};
pub use rtsp::{parse_request, Headers, Limits, Method, ParseError, Request};
pub use session::{Event, SessionError, SessionMachine, SessionState};
