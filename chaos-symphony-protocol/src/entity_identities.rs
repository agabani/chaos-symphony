use serde::{Deserialize, Serialize};

use crate::{Message, MessageCallback, Request, Response};

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Entity Identities Callback.
#[allow(clippy::module_name_repetitions)]
pub type EntityIdentitiesCallback = MessageCallback<EntityIdentitiesResponse>;

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Entity Identities Request.
#[allow(clippy::module_name_repetitions)]
pub type EntityIdentitiesRequest = Message<EntityIdentitiesRequestPayload>;

impl Request<EntityIdentitiesRequestPayload, EntityIdentitiesResponse> for EntityIdentitiesRequest {
    const ENDPOINT: &'static str = "/request/entity_identities";
}

/// Identities Request Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityIdentitiesRequestPayload {}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Entity Identities Response.
#[allow(clippy::module_name_repetitions)]
pub type EntityIdentitiesResponse = Message<EntityIdentitiesResponsePayload>;

impl Response<EntityIdentitiesResponsePayload> for EntityIdentitiesResponse {
    const ENDPOINT: &'static str = "/response/entity_identities";
}

/// Entity Identities Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EntityIdentitiesResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success,
}
