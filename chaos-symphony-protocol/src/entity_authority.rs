use serde::{Deserialize, Serialize};

use crate::{Event, Identity, Message};

/*
 * ============================================================================
 * Event: Entity Client Authority
 * ============================================================================
 */

/// Entity Client Authority Event.
#[allow(clippy::module_name_repetitions)]
pub type EntityClientAuthorityEvent = Message<EntityClientAuthorityEventPayload>;

impl Event<EntityClientAuthorityEventPayload> for EntityClientAuthorityEvent {
    const ENDPOINT: &'static str = "/event/entity_client_authority";
}

/// Entity Client Authority Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityClientAuthorityEventPayload {
    /// Authority Identity.
    pub authority_identity: Identity,

    /// Entity Identity.
    pub entity_identity: Identity,
}

/*
 * ============================================================================
 * Event: Entity Replication Authority
 * ============================================================================
 */

/// Entity Replication Authority Event.
#[allow(clippy::module_name_repetitions)]
pub type EntityReplicationAuthorityEvent = Message<EntityReplicationAuthorityEventPayload>;

impl Event<EntityReplicationAuthorityEventPayload> for EntityReplicationAuthorityEvent {
    const ENDPOINT: &'static str = "/event/entity_replication_authority";
}

/// Entity Replication Authority Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityReplicationAuthorityEventPayload {
    /// Authority Identity.
    pub authority_identity: Identity,

    /// Entity Identity.
    pub entity_identity: Identity,
}

/*
 * ============================================================================
 * Event: Entity Simulation Authority
 * ============================================================================
 */

/// Entity Simulation Authority Event.
#[allow(clippy::module_name_repetitions)]
pub type EntitySimulationAuthorityEvent = Message<EntitySimulationAuthorityEventPayload>;

impl Event<EntitySimulationAuthorityEventPayload> for EntitySimulationAuthorityEvent {
    const ENDPOINT: &'static str = "/event/entity_simulation_authority";
}

/// Entity Simulation Authority Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntitySimulationAuthorityEventPayload {
    /// Authority Identity.
    pub authority_identity: Identity,

    /// Entity Identity.
    pub entity_identity: Identity,
}
