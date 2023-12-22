use bevy::prelude::*;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::{Message, Transformation};

/// Ship Spawn Event.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnEvent = Message<ShipSpawnEventPayload>;

impl ShipSpawnEvent {
    /// Creates a new [`ShipSpawnEvent`].
    #[must_use]
    pub fn new(id: String, payload: ShipSpawnEventPayload) -> Self {
        Self {
            id,
            endpoint: "/event/ship_spawn".to_string(),
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

/// Ship Spawn Event Payload .
#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Serialize)]
pub struct ShipSpawnEventPayload {
    /// Identity.
    pub identity: String,

    /// Client Authority.
    pub client_authority: String,

    /// Server Authority.
    pub server_authority: String,

    /// Transformation.
    pub transformation: Transformation,
}

/// Ship Spawn Request Payload.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnRequest = Message<ShipSpawnRequestPayload>;

impl ShipSpawnRequest {
    /// Creates a new [`ShipSpawnedRequest`].
    #[must_use]
    pub fn new(id: String, payload: ShipSpawnRequestPayload) -> Self {
        Self {
            id,
            endpoint: "/request/ship_spawn".to_string(),
            payload,
        }
    }

    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send(
        self,
        endpoint: &NetworkEndpoint,
    ) -> Result<ShipSpawning, SendError<NetworkSend>> {
        let id = self.id.clone();
        endpoint
            .try_send_blocking(self.into())
            .map(|future| ShipSpawning { id, inner: future })
    }

    /// With Client Authority.
    #[must_use]
    pub fn with_client_authority(mut self, client_authority: String) -> Self {
        self.payload.client_authority = client_authority;
        self
    }

    /// With Server Authority.
    #[must_use]
    pub fn with_server_authority(mut self, server_authority: String) -> Self {
        self.payload.server_authority = server_authority;
        self
    }
}

/// Ship Spawn Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Deserialize, Serialize)]
pub struct ShipSpawnRequestPayload {
    /// Client Authority.
    pub client_authority: String,

    /// Server Authority.
    pub server_authority: String,
}

/// Ship Spawn Response.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnResponse = Message<ShipSpawnResponsePayload>;

impl ShipSpawnResponse {
    /// Creates a new [`ShipSpawnResponse`].
    #[must_use]
    pub fn new(id: String, payload: ShipSpawnResponsePayload) -> Self {
        Self {
            id,
            endpoint: "/response/ship_spawn".to_string(),
            payload,
        }
    }

    /// Creates a new [`ShipSpawnResponse`].
    #[must_use]
    pub fn error(id: String) -> Self {
        Self::new(
            id,
            ShipSpawnResponsePayload {
                success: false,
                identity: String::new(),
                client_authority: String::new(),
                server_authority: String::new(),
                transformation: Transformation::default(),
            },
        )
    }

    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send(self, endpoint: &NetworkEndpoint) -> Result<(), SendError<NetworkSend>> {
        endpoint.try_send_non_blocking(self.into())
    }

    /// With Client Authority.
    #[must_use]
    pub fn with_client_authority(mut self, client_authority: String) -> Self {
        self.payload.client_authority = client_authority;
        self
    }

    /// With Server Authority.
    #[must_use]
    pub fn with_server_authority(mut self, server_authority: String) -> Self {
        self.payload.server_authority = server_authority;
        self
    }
}

/// Ship Spawn Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Serialize)]
pub struct ShipSpawnResponsePayload {
    /// Success.
    pub success: bool,

    /// Identity.
    pub identity: String,

    /// Client Authority.
    pub client_authority: String,

    /// Server Authority.
    pub server_authority: String,

    /// Transformation.
    pub transformation: Transformation,
}

/// Ship Spawning.
#[derive(Component)]
pub struct ShipSpawning {
    /// Id.
    pub id: String,

    inner: Future<chaos_symphony_network::Message>,
}

impl ShipSpawning {
    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<ShipSpawnResponse, PollError>> {
        self.inner.try_poll().map(|r| r.map(Into::into))
    }
}
