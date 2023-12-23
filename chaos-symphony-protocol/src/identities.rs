use bevy::prelude::*;
use bevy::utils::Uuid;
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkSend};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::SendError;

use crate::Message;

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Identities Callback.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Component)]
pub struct IdentitiesCallback {
    /// Id.
    pub id: Uuid,

    future: Future<chaos_symphony_network::Message>,
}

impl IdentitiesCallback {
    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<IdentitiesResponse, PollError>> {
        self.future.try_poll().map(|result| result.map(Into::into))
    }
}

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Identities Request.
#[allow(clippy::module_name_repetitions)]
pub type IdentitiesRequest = Message<IdentitiesRequestPayload>;

impl IdentitiesRequest {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/request/identities";

    /// Creates a new [`IdentitiesRequest`].
    #[must_use]
    pub fn new(id: Uuid, payload: IdentitiesRequestPayload) -> Self {
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
    ) -> Result<IdentitiesCallback, SendError<NetworkSend>> {
        let id = self.id;
        endpoint
            .try_send_blocking(self.into())
            .map(|future| IdentitiesCallback { id, future })
    }
}

/// Identities Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdentitiesRequestPayload {}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Identities Response.
#[allow(clippy::module_name_repetitions)]
pub type IdentitiesResponse = Message<IdentitiesResponsePayload>;

impl IdentitiesResponse {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/response/identities";

    /// Creates a new [`IdentitiesResponsePayload`].
    #[must_use]
    pub fn new(id: Uuid, payload: IdentitiesResponsePayload) -> Self {
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

/// Identities Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum IdentitiesResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success,
}
