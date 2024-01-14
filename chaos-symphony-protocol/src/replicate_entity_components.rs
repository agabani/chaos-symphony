use serde::{Deserialize, Serialize};

use crate::{Identity, Message, MessageCallback, Request, Response};

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Replicate Entity Component Callback.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateEntityComponentsCallback = MessageCallback<ReplicateEntityComponentsResponse>;

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Replicate Entity Component Request.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateEntityComponentsRequest = Message<ReplicateEntityComponentsRequestPayload>;

impl Request<ReplicateEntityComponentsRequestPayload, ReplicateEntityComponentsResponse>
    for ReplicateEntityComponentsRequest
{
    const ENDPOINT: &'static str = "/request/replicate_entity_components";
}

/// Replicate Entity Component Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplicateEntityComponentsRequestPayload {
    /// Entity Identity.
    pub entity_identity: Identity,
}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Replicate Entity Component Response.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateEntityComponentsResponse = Message<ReplicateEntityComponentsResponsePayload>;

impl Response<ReplicateEntityComponentsResponsePayload> for ReplicateEntityComponentsResponse {
    const ENDPOINT: &'static str = "/response/replicate_entity_components";
}

/// Replicate Entity Component Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ReplicateEntityComponentsResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success,
}
