use serde::{Deserialize, Serialize};

use crate::{Event, Identity, Message};

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Identity Event.
#[allow(clippy::module_name_repetitions)]
pub type EntityIdentityEvent = Message<EntityIdentityEventPayload>;

impl Event<EntityIdentityEventPayload> for EntityIdentityEvent {
    const ENDPOINT: &'static str = "/event/entity_identity";
}

/// Identity Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityIdentityEventPayload {
    /// Inner.
    pub inner: Identity,
}
