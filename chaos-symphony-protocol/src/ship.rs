use bevy::utils::Uuid;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::{Identity, Message};

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Ship Event.
#[allow(clippy::module_name_repetitions)]
pub type ShipEvent = Message<ShipEventPayload>;

impl ShipEvent {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/event/ship";

    /// Creates a new [`ShipEvent`].
    #[must_use]
    pub fn new(id: Uuid, payload: ShipEventPayload) -> Self {
        Self {
            id,
            endpoint: Self::ENDPOINT.to_string(),
            payload,
        }
    }

    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send(self, endpoint: &NetworkEndpoint) -> Result<(), SendError<NetworkSend>> {
        endpoint.try_send_non_blocking(self.into())
    }
}

/// Ship Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipEventPayload {
    /// Identity.
    pub identity: Identity,
}
