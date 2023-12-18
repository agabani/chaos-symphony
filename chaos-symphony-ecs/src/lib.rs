#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony ECS

/// Authority.
pub mod authority;
/// Entity.
pub mod entity;
/// Network Authenticate.
pub mod network_authenticate;
/// Network Connect.
pub mod network_connect;
/// Network Disconnect.
pub mod network_disconnect;
/// Network Keep Alive.
pub mod network_keep_alive;
/// Ship.
pub mod ship;
