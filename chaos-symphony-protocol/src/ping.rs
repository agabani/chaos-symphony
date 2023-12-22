use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::Message;

/// Ping Event.
#[allow(clippy::module_name_repetitions)]
pub type PingEvent = Message<PingEventPayload>;

impl PingEvent {
    /// Creates a new [`PingEvent`].
    #[must_use]
    pub fn new(id: String) -> Self {
        Self {
            id,
            endpoint: "/event/ping".to_string(),
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
#[derive(Deserialize, Serialize)]
pub struct PingEventPayload;
