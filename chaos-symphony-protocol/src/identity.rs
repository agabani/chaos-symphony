use bevy::prelude::*;
use bevy::utils::Uuid;
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

/// Identity Callback.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Component)]
pub struct IdentityCallback {
    /// Id.
    pub id: Uuid,

    future: Future<chaos_symphony_network::Message>,
}

impl IdentityCallback {
    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<IdentityResponse, PollError>> {
        self.future.try_poll().map(|result| result.map(Into::into))
    }
}

/*
 * ============================================================================
 * Event
 * ============================================================================
 */

/// Identity Event.
#[allow(clippy::module_name_repetitions)]
pub type IdentityEvent = Message<IdentityEventPayload>;

impl IdentityEvent {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/event/identity";

    /// Creates a new [`IdentityEvent`].
    #[must_use]
    pub fn new(id: Uuid, payload: IdentityEventPayload) -> Self {
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

/// Identity Event Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdentityEventPayload {
    /// Identity.
    pub identity: Identity,
}

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Identity Request.
#[allow(clippy::module_name_repetitions)]
pub type IdentityRequest = Message<IdentityRequestPayload>;

impl IdentityRequest {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/request/identity";

    /// Creates a new [`IdentityRequest`].
    #[must_use]
    pub fn new(id: Uuid, payload: IdentityRequestPayload) -> Self {
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
    ) -> Result<IdentityCallback, SendError<NetworkSend>> {
        let id = self.id;
        endpoint
            .try_send_blocking(self.into())
            .map(|future| IdentityCallback { id, future })
    }
}

/// Identity Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdentityRequestPayload {}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Identity Response.
#[allow(clippy::module_name_repetitions)]
pub type IdentityResponse = Message<IdentityResponsePayload>;

impl IdentityResponse {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/response/identity";

    /// Creates a new [`IdentityResponsePayload`].
    #[must_use]
    pub fn new(id: Uuid, payload: IdentityResponsePayload) -> Self {
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

/// Identity Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum IdentityResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success,
}
