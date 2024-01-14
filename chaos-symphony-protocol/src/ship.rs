use serde::{Deserialize, Serialize};

use crate::{Event, Identity, Message};

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Ship Event.
#[allow(clippy::module_name_repetitions)]
pub type ShipEvent = Message<ShipEventPayload>;

impl Event<ShipEventPayload> for ShipEvent {
    const ENDPOINT: &'static str = "/event/ship";
}

/// Ship Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipEventPayload {
    /// Entity Identity.
    pub entity_identity: Identity,
}
