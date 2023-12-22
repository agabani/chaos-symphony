use bevy::prelude::*;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::Message;

/// Authenticate Request.
#[allow(clippy::module_name_repetitions)]
pub type AuthenticateRequest = Message<AuthenticateRequestPayload>;

impl AuthenticateRequest {
    /// Creates a new [`AuthenticateRequest`].
    #[must_use]
    pub fn new(id: String, payload: AuthenticateRequestPayload) -> Self {
        Self {
            id,
            endpoint: "/request/authenticate".to_string(),
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
    ) -> Result<Authenticating, SendError<NetworkSend>> {
        let id = self.id.clone();
        endpoint
            .try_send_blocking(self.into())
            .map(|future| Authenticating { id, inner: future })
    }
}

/// Authenticate Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Serialize)]
pub struct AuthenticateRequestPayload {
    /// Identity.
    pub identity: String,
}

/// Authenticate Response.
#[allow(clippy::module_name_repetitions)]
pub type AuthenticateResponse = Message<AuthenticateResponsePayload>;

impl AuthenticateResponse {
    /// Creates a new [`AuthenticateResponse`].
    #[must_use]
    pub fn new(id: String, payload: AuthenticateResponsePayload) -> Self {
        Self {
            id,
            endpoint: "/response/authenticate".to_string(),
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

/// Authenticate Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize, Serialize)]
pub struct AuthenticateResponsePayload {
    /// Success.
    pub success: bool,

    /// Identity.
    pub identity: String,
}

/// Authenticating.
#[derive(Component)]
pub struct Authenticating {
    /// Id.
    pub id: String,

    inner: Future<chaos_symphony_network::Message>,
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
