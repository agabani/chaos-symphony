use bevy::prelude::*;

use crate::{
    transform::Transformation,
    types::{ClientAuthority, Identity, ServerAuthority},
};

/// Ship.
#[derive(Component)]
pub struct Ship;

/// Ship Bundle.
#[allow(clippy::module_name_repetitions)]
#[derive(Bundle)]
pub struct ShipBundle {
    /// Ship.
    pub ship: Ship,

    /// Identity.
    pub identity: Identity,

    /// Client Authority.
    pub client_authority: ClientAuthority,

    /// Server Authority.
    pub server_authority: ServerAuthority,

    /// Transform.
    pub transformation: Transformation,
}
