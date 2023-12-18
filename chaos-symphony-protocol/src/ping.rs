use std::collections::HashMap;

use chaos_symphony_network::Payload;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};

/// Ping
pub struct Ping {
    /// Id.
    pub id: String,
}

impl Ping {
    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send(
        self,
        endpoint: &NetworkEndpoint,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<NetworkSend>> {
        endpoint.try_send_non_blocking(self.into())
    }
}

impl From<Payload> for Ping {
    fn from(value: Payload) -> Self {
        Self { id: value.id }
    }
}

impl From<Ping> for Payload {
    fn from(value: Ping) -> Self {
        Self {
            id: value.id,
            endpoint: "/event/ping".to_string(),
            properties: HashMap::new(),
        }
    }
}
