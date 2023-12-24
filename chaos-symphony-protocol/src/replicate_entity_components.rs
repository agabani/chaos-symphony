use serde::{Deserialize, Serialize};

use crate::{Identity, Message, MessageCallback, Request, Response};

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Entity Identities Callback.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateEntityComponentsCallback = MessageCallback<ReplicateEntityComponentsResponse>;

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Entity Identities Request.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateEntityComponentsRequest = Message<ReplicateEntityComponentsRequestPayload>;

impl Request<ReplicateEntityComponentsRequestPayload, ReplicateEntityComponentsResponse>
    for ReplicateEntityComponentsRequest
{
    const ENDPOINT: &'static str = "/request/replicate_entity_components";
}

/// Identities Request Payload.
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

/// Entity Identities Response.
#[allow(clippy::module_name_repetitions)]
pub type ReplicateEntityComponentsResponse = Message<ReplicateEntityComponentsResponsePayload>;

impl Response<ReplicateEntityComponentsResponsePayload> for ReplicateEntityComponentsResponse {
    const ENDPOINT: &'static str = "/response/replicate_entity_components";
}

/// Entity Identities Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ReplicateEntityComponentsResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success,
}
