use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::{Identity, Message};

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Authenticate Callback.
#[derive(Debug, Component)]
pub struct AuthenticateCallback {
    /// Id.
    pub id: Uuid,

    future: Future<chaos_symphony_network::Message>,
}

impl AuthenticateCallback {
    /// Id.
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<AuthenticateResponse, PollError>> {
        self.future.try_poll().map(|result| result.map(Into::into))
    }
}

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Authenticate Request.
#[allow(clippy::module_name_repetitions)]
pub type AuthenticateRequest = Message<AuthenticateRequestPayload>;

impl AuthenticateRequest {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/request/authenticate";

    /// Creates a new [`AuthenticateRequest`].
    #[must_use]
    pub fn new(id: Uuid, payload: AuthenticateRequestPayload) -> Self {
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
    ) -> Result<AuthenticateCallback, SendError<NetworkSend>> {
        let id = self.id;
        endpoint
            .try_send_blocking(self.into())
            .map(|future| AuthenticateCallback { id, future })
    }
}

/// Authenticate Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthenticateRequestPayload {
    /// Identity.
    pub identity: Identity,
}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Authenticate Response.
#[allow(clippy::module_name_repetitions)]
pub type AuthenticateResponse = Message<AuthenticateResponsePayload>;

impl AuthenticateResponse {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/response/authenticate";

    /// Creates a new [`AuthenticateResponse`].
    #[must_use]
    pub fn new(id: Uuid, payload: AuthenticateResponsePayload) -> Self {
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

/// Authenticate Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AuthenticateResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success {
        /// Identity.
        identity: Identity,
    },
}
