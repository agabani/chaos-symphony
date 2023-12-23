#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony ECS

/// Authority.
pub mod authority;
/// Identities;
pub mod identities;
/// Identity.
pub mod identity;
/// Network.
pub mod network;
/// Network Authenticate.
pub mod network_authenticate;
/// Network Connect.
pub mod network_connect;
/// Network Disconnect.
pub mod network_disconnect;
/// Network Keep Alive.
pub mod network_keep_alive;
/// Replicate.
pub mod replicate;
/// Ship.
pub mod ship;
/// Ship Spawn.
pub mod ship_spawn;
/// Transform.
pub mod transform;
/// Types.
pub mod types;
