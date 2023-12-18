use std::collections::HashMap;

use bevy::prelude::*;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network::Payload;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};

/// Ship Spawn Request
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ShipSpawnRequest {
    /// Id.
    pub id: String,
}

impl ShipSpawnRequest {
    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send(
        self,
        endpoint: &NetworkEndpoint,
    ) -> Result<ShipSpawning, tokio::sync::mpsc::error::SendError<NetworkSend>> {
        endpoint
            .try_send_blocking(self.into())
            .map(|future| ShipSpawning { inner: future })
    }
}

impl From<Payload> for ShipSpawnRequest {
    fn from(value: Payload) -> Self {
        Self { id: value.id }
    }
}

impl From<ShipSpawnRequest> for Payload {
    fn from(value: ShipSpawnRequest) -> Self {
        Self {
            id: value.id,
            endpoint: "/request/ship_spawn".to_string(),
            properties: HashMap::new(),
        }
    }
}

/// Ship Spawn Response.
#[allow(clippy::module_name_repetitions)]
pub struct ShipSpawnResponse {
    /// Id.
    pub id: String,

    /// Success.
    pub success: bool,

    /// Identity.
    pub identity: String,
}

impl From<Payload> for ShipSpawnResponse {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            success: value.properties.remove("success").unwrap().parse().unwrap(),
            identity: value.properties.remove("identity").unwrap(),
        }
    }
}

impl From<ShipSpawnResponse> for Payload {
    fn from(value: ShipSpawnResponse) -> Self {
        Self {
            id: value.id,
            endpoint: "/response/ship_spawn".to_string(),
            properties: HashMap::from([("success".to_string(), value.success.to_string())]),
        }
    }
}

/// Ship Spawning.
#[derive(Component)]
pub struct ShipSpawning {
    inner: Future<Payload>,
}

impl ShipSpawning {
    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<ShipSpawnResponse, PollError>> {
        self.inner.try_poll().map(|r| r.map(Into::into))
    }
}

/// Ship Spawn Response.
pub struct ShipSpawned {
    /// Id.
    pub id: String,

    /// Identity.
    pub identity: String,
}
