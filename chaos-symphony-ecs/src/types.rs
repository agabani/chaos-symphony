use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};

/// Identity.
#[derive(Debug, Clone, PartialEq, Eq, Component, Reflect)]
pub struct Identity {
    id: Uuid,

    noun: String,
}

impl Identity {
    /// Creates a new [`Identity`].
    #[must_use]
    pub fn new(noun: String, id: Uuid) -> Self {
        Self { id, noun }
    }

    /// Id.
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Noun.
    #[must_use]
    pub fn noun(&self) -> &str {
        &self.noun
    }
}

impl From<chaos_symphony_protocol::Identity> for Identity {
    fn from(value: chaos_symphony_protocol::Identity) -> Self {
        Self {
            id: value.id,
            noun: value.noun,
        }
    }
}

impl From<Identity> for chaos_symphony_protocol::Identity {
    fn from(value: Identity) -> Self {
        Self {
            id: value.id,
            noun: value.noun,
        }
    }
}

/// Transformation.
#[derive(Clone, Copy, Component, Reflect)]
pub struct Transformation {
    /// Orientation.
    pub orientation: DQuat,

    /// Position.
    pub position: DVec3,
}

impl From<chaos_symphony_protocol::Transformation> for Transformation {
    fn from(value: chaos_symphony_protocol::Transformation) -> Self {
        Self {
            orientation: value.orientation.into(),
            position: value.position.into(),
        }
    }
}

impl From<Transformation> for chaos_symphony_protocol::Transformation {
    fn from(value: Transformation) -> Self {
        Self {
            orientation: value.orientation.into(),
            position: value.position.into(),
        }
    }
}

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
