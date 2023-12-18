use std::collections::HashMap;

use bevy::prelude::*;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network::Payload;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};

/// Authenticate Request
#[allow(clippy::module_name_repetitions)]
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
        let id = self.id.clone();
        endpoint
            .try_send_blocking(self.into())
            .map(|future| Authenticating { id, inner: future })
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
#[allow(clippy::module_name_repetitions)]
pub struct AuthenticateResponse {
    /// Id.
    pub id: String,

    /// Success.
    pub success: bool,

    /// Identity.
    pub identity: String,
}

impl From<Payload> for AuthenticateResponse {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            success: value.properties.remove("success").unwrap().parse().unwrap(),
            identity: value.properties.remove("identity").unwrap(),
        }
    }
}

impl From<AuthenticateResponse> for Payload {
    fn from(value: AuthenticateResponse) -> Self {
        Self {
            id: value.id,
            endpoint: "/response/authenticate".to_string(),
            properties: HashMap::from([
                ("success".to_string(), value.success.to_string()),
                ("identity".to_string(), value.identity),
            ]),
        }
    }
}

/// Authenticating
#[derive(Component)]
pub struct Authenticating {
    /// Id.
    pub id: String,

    inner: Future<Payload>,
}

impl Authenticating {
    /// Id.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<AuthenticateResponse, PollError>> {
        self.inner.try_poll().map(|r| r.map(Into::into))
    }
}
