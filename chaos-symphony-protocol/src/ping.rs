use bevy::utils::Uuid;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::Message;

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Ping Event.
#[allow(clippy::module_name_repetitions)]
pub type PingEvent = Message<PingEventPayload>;

impl PingEvent {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/event/ping";

    /// Creates a new [`PingEvent`].
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            endpoint: Self::ENDPOINT.to_string(),
            payload: PingEventPayload,
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

/// Ping Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PingEventPayload;
