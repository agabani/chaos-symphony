use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};

/*
 * ============================================================================
 * Identity
 * ============================================================================
 */

/// Identity.
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Identity {
    /// Id.
    pub id: Uuid,

    /// Noun.
    pub noun: String,
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

impl PartialEq<chaos_symphony_protocol::Identity> for Identity {
    fn eq(&self, other: &chaos_symphony_protocol::Identity) -> bool {
        self.id == other.id && self.noun == other.noun
    }
}

impl PartialEq<Identity> for chaos_symphony_protocol::Identity {
    fn eq(&self, other: &Identity) -> bool {
        self.id == other.id && self.noun == other.noun
    }
}

/*
 * ============================================================================
 * Entity
 * ============================================================================
 */

/// Entity Identity.
#[derive(Debug, Clone, PartialEq, Eq, Component, Reflect)]
pub struct EntityIdentity {
    /// Inner.
    pub inner: Identity,
}

/// Entity Authority.
pub trait EntityAuthority {
    /// Identity.
    fn identity(&self) -> &Identity;
}

/// Entity Client Authority.
#[derive(Debug, Clone, PartialEq, Eq, Component, Reflect)]
pub struct EntityClientAuthority {
    /// Identity.
    pub identity: Identity,
}

impl EntityAuthority for EntityClientAuthority {
    fn identity(&self) -> &Identity {
        &self.identity
    }
}

/// Entity Replication Authority.
#[derive(Debug, Clone, PartialEq, Eq, Component, Reflect)]
pub struct EntityReplicationAuthority {
    /// Identity.
    pub identity: Identity,
}

impl EntityAuthority for EntityReplicationAuthority {
    fn identity(&self) -> &Identity {
        &self.identity
    }
}

/// Entity Simulation Authority.
#[derive(Debug, Clone, PartialEq, Eq, Component, Reflect)]
pub struct EntitySimulationAuthority {
    /// Identity.
    pub identity: Identity,
}

impl EntityAuthority for EntitySimulationAuthority {
    fn identity(&self) -> &Identity {
        &self.identity
    }
}

/*
 * ============================================================================
 * Network
 * ============================================================================
 */

/// Network Identity.
#[derive(Debug, Clone, PartialEq, Eq, Component, Resource, Reflect)]
pub struct NetworkIdentity {
    /// Inner.
    pub inner: Identity,
}

/// Network Client Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Reflect)]
pub struct NetworkClientAuthority;

/// Network Replication Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Reflect)]
pub struct NetworkReplicationAuthority;

/// Network Server Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Reflect)]
pub struct NetworkServerAuthority;

/*
 * ============================================================================
 * Replicate
 * ============================================================================
 */

/// Replicate Sink.
#[derive(Debug, Clone, PartialEq, Eq, Component, Reflect)]
pub struct ReplicateSink;

/// Replicate Source.
#[derive(Debug, Clone, PartialEq, Eq, Component, Reflect)]
pub struct ReplicateSource;

/*
 * ============================================================================
 * Role
 * ============================================================================
 */

/// Role.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy)]
pub enum Role {
    /// Client.
    Client,

    /// Replication.
    Replication,

    /// Simulation.
    Simulation,
}

/*
 * ============================================================================
 * Transformation
 * ============================================================================
 */

/// Transformation.
#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect)]
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

/*
 * ============================================================================
 * Trust
 * ============================================================================
 */

/// Trusted.
#[derive(Debug, Clone, Event)]
pub struct Trusted<T> {
    /// Inner.
    pub inner: T,
}

/// Untrusted.
#[derive(Debug, Clone, Event)]
pub struct Untrusted<T> {
    /// Inner.
    pub inner: T,
}
