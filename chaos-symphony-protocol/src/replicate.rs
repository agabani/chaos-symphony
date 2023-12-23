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

/// Replicate Callback.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Component)]
pub struct ReplicateCallback {
    /// Id.
    pub id: Uuid,

    future: Future<chaos_symphony_network::Message>,
}

impl ReplicateCallback {
    /// Try poll.
    pub fn try_poll(&self) -> Poll<Result<ReplicateResponse, PollError>> {
        self.future.try_poll().map(|result| result.map(Into::into))
    }
}

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Replicate Request.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateRequest = Message<ReplicateRequestPayload>;

impl ReplicateRequest {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/request/replicate";

    /// Creates a new [`ReplicateRequest`].
    #[must_use]
    pub fn new(id: Uuid, payload: ReplicateRequestPayload) -> Self {
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
    ) -> Result<ReplicateCallback, SendError<NetworkSend>> {
        let id = self.id;
        endpoint
            .try_send_blocking(self.into())
            .map(|future| ReplicateCallback { id, future })
    }
}

/// Replicate Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplicateRequestPayload {
    /// Identity.
    pub identity: Identity,
}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Replicate Response.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateResponse = Message<ReplicateResponsePayload>;

impl ReplicateResponse {
    /// Endpoint.
    pub const ENDPOINT: &'static str = "/response/replicate";

    /// Creates a new [`ReplicateResponsePayload`].
    #[must_use]
    pub fn new(id: Uuid, payload: ReplicateResponsePayload) -> Self {
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

/// Replicate Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ReplicateResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success,
}
