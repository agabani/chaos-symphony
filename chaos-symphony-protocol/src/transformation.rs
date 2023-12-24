use serde::{Deserialize, Serialize};

use crate::{Event, Identity, Message, Transformation};

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Transformation Event.
#[allow(clippy::module_name_repetitions)]
pub type TransformationEvent = Message<TransformationEventPayload>;

impl Event<TransformationEventPayload> for TransformationEvent {
    const ENDPOINT: &'static str = "/event/transformation";
}

/// Transformation Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransformationEventPayload {
    /// Entity Identity.
    pub entity_identity: Identity,

    /// Transformation.
    pub transformation: Transformation,
}
