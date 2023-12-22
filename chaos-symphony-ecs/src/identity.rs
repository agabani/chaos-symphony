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
