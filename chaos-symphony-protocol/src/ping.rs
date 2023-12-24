use serde::{Deserialize, Serialize};

use crate::{Event, Message};

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Ping Event.
#[allow(clippy::module_name_repetitions)]
pub type PingEvent = Message<PingEventPayload>;

impl Event<PingEventPayload> for PingEvent {
    const ENDPOINT: &'static str = "/event/ping";
}

/// Ping Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PingEventPayload;
