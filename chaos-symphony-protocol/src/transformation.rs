use bevy::utils::Uuid;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::{Identity, Message, Transformation};

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Transformation Event.
#[allow(clippy::module_name_repetitions)]
pub type TransformationEvent = Message<TransformationEventPayload>;

impl TransformationEvent {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/event/transformation";

    /// Creates a new [`TransformationEvent`].
    #[must_use]
    pub fn new(id: Uuid, payload: TransformationEventPayload) -> Self {
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

/// Transformation Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransformationEventPayload {
    /// Identity.
    pub identity: Identity,

    /// Transformation.
    pub transformation: Transformation,
}
