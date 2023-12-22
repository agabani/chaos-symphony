use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::{Identity, Message, Transformation};

/// Ship Spawn Event.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnEvent = Message<ShipSpawnEventPayload>;

impl ShipSpawnEvent {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/event/ship_spawn";

    /// Creates a new [`ShipSpawnEvent`].
    #[must_use]
    pub fn new(id: Uuid, payload: ShipSpawnEventPayload) -> Self {
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

/// Ship Spawn Event Payload .
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipSpawnEventPayload {
    /// Identity.
    pub identity: Identity,

    /// Client Authority.
    pub client_authority: Identity,

    /// Server Authority.
    pub server_authority: Identity,

    /// Transformation.
    pub transformation: Transformation,
}

/// Ship Spawn Request Payload.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnRequest = Message<ShipSpawnRequestPayload>;

impl ShipSpawnRequest {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/request/ship_spawn";

    /// Creates a new [`ShipSpawnedRequest`].
    #[must_use]
    pub fn new(id: Uuid, payload: ShipSpawnRequestPayload) -> Self {
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
    pub fn try_send(
        self,
        endpoint: &NetworkEndpoint,
    ) -> Result<ShipSpawning, SendError<NetworkSend>> {
        let id = self.id;
        endpoint
            .try_send_blocking(self.into())
            .map(|future| ShipSpawning { id, future })
    }
}

/// Ship Spawn Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipSpawnRequestPayload {
    /// Client Authority.
    pub client_authority: Option<Identity>,

    /// Server Authority.
    pub server_authority: Option<Identity>,
}

/// Ship Spawn Response.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnResponse = Message<ShipSpawnResponsePayload>;

impl ShipSpawnResponse {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/response/ship_spawn";

    /// Creates a new [`ShipSpawnResponse`].
    #[must_use]
    pub fn new(id: Uuid, payload: ShipSpawnResponsePayload) -> Self {
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

/// Ship Spawn Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ShipSpawnResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success(ShipSpawnResponsePayloadSuccess),
}

/// Ship Spawn Response Payload Success.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipSpawnResponsePayloadSuccess {
    /// Identity.
    pub identity: Identity,

    /// Client Authority.
    pub client_authority: Identity,

    /// Server Authority.
    pub server_authority: Identity,

    /// Transformation.
    pub transformation: Transformation,
}

/// Ship Spawning.
#[derive(Debug, Component)]
pub struct ShipSpawning {
    /// Id.
    pub id: Uuid,

    future: Future<chaos_symphony_network::Message>,
}

impl ShipSpawning {
    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<ShipSpawnResponse, PollError>> {
        self.future.try_poll().map(|result| result.map(Into::into))
    }
}
