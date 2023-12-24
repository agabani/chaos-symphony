use std::marker::PhantomData;

use bevy::{prelude::*, utils::Uuid};

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

/// Client Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component, Reflect)]
pub struct ClientAuthority {
    /// Identity.
    pub identity: Identity,
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

/// Entity Identity.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component, Reflect)]
pub struct EntityIdentity {
    inner: Identity,
}

/// Server Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component, Reflect)]
pub struct ServerAuthority {
    /// Identity.
    pub identity: Identity,
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

/// Replicate.
#[derive(Debug, Clone, Component)]
pub struct Replicate<T> {
    /// Identity.
    pub identity: Identity,

    marker: PhantomData<T>,
}

impl<T> Replicate<T> {
    /// Creates a new [`Replicate`].
    pub fn new(identity: Identity) -> Self {
        Self {
            identity,
            marker: PhantomData,
        }
    }
}
