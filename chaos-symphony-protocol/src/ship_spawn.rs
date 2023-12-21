use std::collections::HashMap;

use bevy::prelude::*;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network::Payload;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};

/// Ship Spawn Event.
#[allow(clippy::module_name_repetitions)]
pub struct ShipSpawnEvent {
    /// Id.
    pub id: String,

    /// Identity.
    pub identity: String,

    /// Client Authority.
    pub client_authority: String,

    /// Server Authority.
    pub server_authority: String,
}

impl ShipSpawnEvent {
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

impl From<Payload> for ShipSpawnEvent {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            identity: value.properties.remove("identity").unwrap(),
            client_authority: value.properties.remove("client_authority").unwrap(),
            server_authority: value.properties.remove("server_authority").unwrap(),
        }
    }
}

impl From<ShipSpawnEvent> for Payload {
    fn from(value: ShipSpawnEvent) -> Self {
        Self {
            id: value.id,
            endpoint: "/event/ship_spawn".to_string(),
            properties: HashMap::from([
                ("identity".to_string(), value.identity),
                ("client_authority".to_string(), value.client_authority),
                ("server_authority".to_string(), value.server_authority),
            ]),
        }
    }
}

/// Ship Spawn Request
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct ShipSpawnRequest {
    /// Id.
    pub id: String,

    /// Client Authority.
    pub client_authority: String,

    /// Server Authority.
    pub server_authority: String,
}

impl ShipSpawnRequest {
    /// Creates a new [`ShipSpawnRequest`].
    #[must_use]
    pub fn new(id: String) -> Self {
        Self {
            id,
            client_authority: String::new(),
            server_authority: String::new(),
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
    ) -> Result<ShipSpawning, tokio::sync::mpsc::error::SendError<NetworkSend>> {
        let id = self.id.clone();
        endpoint
            .try_send_blocking(self.into())
            .map(|future| ShipSpawning { id, inner: future })
    }

    /// With Client Authority.
    #[must_use]
    pub fn with_client_authority(mut self, client_authority: String) -> Self {
        self.client_authority = client_authority;
        self
    }

    /// With Server Authority.
    #[must_use]
    pub fn with_server_authority(mut self, server_authority: String) -> Self {
        self.server_authority = server_authority;
        self
    }
}

impl From<Payload> for ShipSpawnRequest {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            client_authority: value.properties.remove("client_authority").unwrap(),
            server_authority: value.properties.remove("server_authority").unwrap(),
        }
    }
}

impl From<ShipSpawnRequest> for Payload {
    fn from(value: ShipSpawnRequest) -> Self {
        Self {
            id: value.id,
            endpoint: "/request/ship_spawn".to_string(),
            properties: HashMap::from([
                ("client_authority".to_string(), value.client_authority),
                ("server_authority".to_string(), value.server_authority),
            ]),
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

    /// Client Authority.
    pub client_authority: String,

    /// Server Authority.
    pub server_authority: String,
}

impl ShipSpawnResponse {
    /// Creates a new [`ShipSpawnResponse`].
    #[must_use]
    pub fn error(id: String) -> Self {
        Self {
            id,
            success: false,
            identity: String::new(),
            client_authority: String::new(),
            server_authority: String::new(),
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
    ) -> Result<(), tokio::sync::mpsc::error::SendError<NetworkSend>> {
        endpoint.try_send_non_blocking(self.into())
    }

    /// With Client Authority.
    #[must_use]
    pub fn with_client_authority(mut self, client_authority: String) -> Self {
        self.client_authority = client_authority;
        self
    }

    /// With Server Authority.
    #[must_use]
    pub fn with_server_authority(mut self, server_authority: String) -> Self {
        self.server_authority = server_authority;
        self
    }
}

impl From<Payload> for ShipSpawnResponse {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            success: value.properties.remove("success").unwrap().parse().unwrap(),
            identity: value.properties.remove("identity").unwrap(),
            client_authority: value.properties.remove("client_authority").unwrap(),
            server_authority: value.properties.remove("server_authority").unwrap(),
        }
    }
}

impl From<ShipSpawnResponse> for Payload {
    fn from(value: ShipSpawnResponse) -> Self {
        Self {
            id: value.id,
            endpoint: "/response/ship_spawn".to_string(),
            properties: HashMap::from([
                ("success".to_string(), value.success.to_string()),
                ("identity".to_string(), value.identity),
                ("client_authority".to_string(), value.client_authority),
                ("server_authority".to_string(), value.server_authority),
            ]),
        }
    }
}

/// Ship Spawning.
#[derive(Component)]
pub struct ShipSpawning {
    /// Id.
    pub id: String,

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
