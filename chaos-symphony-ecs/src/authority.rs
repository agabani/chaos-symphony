use bevy::prelude::*;

use crate::types::Identity;

/// Client Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component, Reflect)]
pub struct ClientAuthority {
    identity: Identity,
}

impl ClientAuthority {
    /// Creates a new [`ClientAuthority`].
    #[must_use]
    pub fn new(identity: Identity) -> Self {
        Self { identity }
    }

    /// Identity.
    #[must_use]
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
}

/// Server Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component, Reflect)]
pub struct ServerAuthority {
    identity: Identity,
}

impl ServerAuthority {
    /// Creates a new [`ServerAuthority`].
    #[must_use]
    pub fn new(identity: Identity) -> Self {
        Self { identity }
    }

    /// Identity.
    #[must_use]
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
}
