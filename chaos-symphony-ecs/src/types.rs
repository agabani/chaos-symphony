use bevy::{
    ecs::system::EntityCommands,
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_protocol::Event as _;

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

/*
 * ============================================================================
 * Entity: Entity Client Authority
 * ============================================================================
 */

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

impl ReplicateComponent for EntityClientAuthority {
    type Message = chaos_symphony_protocol::EntityClientAuthorityEvent;

    fn to_message(&self, entity_identity: &EntityIdentity) -> Self::Message {
        chaos_symphony_protocol::EntityClientAuthorityEvent::message(
            Uuid::new_v4(),
            chaos_symphony_protocol::EntityClientAuthorityEventPayload {
                authority_identity: self.identity.clone().into(),
                entity_identity: entity_identity.inner.clone().into(),
            },
        )
    }
}

impl ReplicateEvent for chaos_symphony_protocol::EntityClientAuthorityEvent {
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity {
        &self.payload.entity_identity
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn insert_bundle(&self, mut commands: EntityCommands<'_, '_, '_>) {
        let component = EntityClientAuthority {
            identity: self.payload.authority_identity.clone().into(),
        };
        commands.insert(component);
    }

    fn source_identity(&self) -> Option<&chaos_symphony_protocol::Identity> {
        self.header.source_identity.as_ref()
    }
}

/*
 * ============================================================================
 * Entity: Entity Replication Authority
 * ============================================================================
 */

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

impl ReplicateComponent for EntityReplicationAuthority {
    type Message = chaos_symphony_protocol::EntityReplicationAuthorityEvent;

    fn to_message(&self, entity_identity: &EntityIdentity) -> Self::Message {
        chaos_symphony_protocol::EntityReplicationAuthorityEvent::message(
            Uuid::new_v4(),
            chaos_symphony_protocol::EntityReplicationAuthorityEventPayload {
                authority_identity: self.identity.clone().into(),
                entity_identity: entity_identity.inner.clone().into(),
            },
        )
    }
}

impl ReplicateEvent for chaos_symphony_protocol::EntityReplicationAuthorityEvent {
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity {
        &self.payload.entity_identity
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn insert_bundle(&self, mut commands: EntityCommands<'_, '_, '_>) {
        let component = EntityReplicationAuthority {
            identity: self.payload.authority_identity.clone().into(),
        };
        commands.insert(component);
    }

    fn source_identity(&self) -> Option<&chaos_symphony_protocol::Identity> {
        self.header.source_identity.as_ref()
    }
}

/*
 * ============================================================================
 * Entity: Entity Simulation Authority
 * ============================================================================
 */

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

impl ReplicateComponent for EntitySimulationAuthority {
    type Message = chaos_symphony_protocol::EntitySimulationAuthorityEvent;

    fn to_message(&self, entity_identity: &EntityIdentity) -> Self::Message {
        chaos_symphony_protocol::EntitySimulationAuthorityEvent::message(
            Uuid::new_v4(),
            chaos_symphony_protocol::EntitySimulationAuthorityEventPayload {
                authority_identity: self.identity.clone().into(),
                entity_identity: entity_identity.inner.clone().into(),
            },
        )
    }
}

impl ReplicateEvent for chaos_symphony_protocol::EntitySimulationAuthorityEvent {
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity {
        &self.payload.entity_identity
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn insert_bundle(&self, mut commands: EntityCommands<'_, '_, '_>) {
        let component = EntitySimulationAuthority {
            identity: self.payload.authority_identity.clone().into(),
        };
        commands.insert(component);
    }

    fn source_identity(&self) -> Option<&chaos_symphony_protocol::Identity> {
        self.header.source_identity.as_ref()
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

/// Replicate Component.
pub trait ReplicateComponent {
    /// Message.
    type Message;

    /// To Message.
    fn to_message(&self, entity_identity: &EntityIdentity) -> Self::Message;
}

/// Replicate Event.
pub trait ReplicateEvent {
    /// Entity Identity.
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity;

    /// Id.
    fn id(&self) -> Uuid;

    /// Insert Bundle.
    fn insert_bundle(&self, commands: EntityCommands<'_, '_, '_>);

    /// Source Identity.
    fn source_identity(&self) -> Option<&chaos_symphony_protocol::Identity>;
}

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

impl ReplicateComponent for Transformation {
    type Message = chaos_symphony_protocol::TransformationEvent;

    fn to_message(&self, entity_identity: &EntityIdentity) -> Self::Message {
        chaos_symphony_protocol::TransformationEvent::message(
            Uuid::new_v4(),
            chaos_symphony_protocol::TransformationEventPayload {
                entity_identity: entity_identity.inner.clone().into(),
                transformation: (*self).into(),
            },
        )
    }
}

impl ReplicateEvent for chaos_symphony_protocol::TransformationEvent {
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity {
        &self.payload.entity_identity
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn insert_bundle(&self, mut commands: EntityCommands) {
        let component: Transformation = self.payload.transformation.into();
        commands.insert(component);
    }

    fn source_identity(&self) -> Option<&chaos_symphony_protocol::Identity> {
        self.header.source_identity.as_ref()
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
