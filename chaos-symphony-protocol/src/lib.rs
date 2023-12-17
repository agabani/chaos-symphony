#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Protocol

use std::collections::HashMap;

use bevy::prelude::*;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_bevy_network::{NetworkEndpoint, NetworkSend};
use chaos_symphony_network::Payload;

/// Authenticate Request
pub struct AuthenticateRequest {
    /// Id.
    pub id: String,

    /// Identity.
    pub identity: String,
}

impl AuthenticateRequest {
    /// Try send.
    ///
    /// # Errors
    ///
    /// Will return `Err` if bevy-tokio bridge is disconnected.
    pub fn try_send(
        self,
        endpoint: &NetworkEndpoint,
    ) -> Result<Authenticating, tokio::sync::mpsc::error::SendError<NetworkSend>> {
        endpoint
            .try_send_blocking(self.into())
            .map(|future| Authenticating { inner: future })
    }
}

impl From<Payload> for AuthenticateRequest {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            identity: value.properties.remove("identity").unwrap(),
        }
    }
}

impl From<AuthenticateRequest> for Payload {
    fn from(value: AuthenticateRequest) -> Self {
        Self {
            id: value.id,
            endpoint: "/request/authenticate".to_string(),
            properties: HashMap::from([("identity".to_string(), value.identity)]),
        }
    }
}

/// Authenticate Response.
pub struct AuthenticateResponse {
    /// Id.
    pub id: String,

    /// Success.
    pub success: bool,
}

impl From<Payload> for AuthenticateResponse {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            success: value.properties.remove("success").unwrap().parse().unwrap(),
        }
    }
}

impl From<AuthenticateResponse> for Payload {
    fn from(value: AuthenticateResponse) -> Self {
        Self {
            id: value.id,
            endpoint: "/response/authenticate".to_string(),
            properties: HashMap::from([("success".to_string(), value.success.to_string())]),
        }
    }
}

/// Authenticating
#[derive(Component)]
pub struct Authenticating {
    inner: Future<Payload>,
}

impl Authenticating {
    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<AuthenticateResponse, PollError>> {
        self.inner.try_poll().map(|r| r.map(Into::into))
    }
}

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
