use serde::{Deserialize, Serialize};

use crate::{Message, MessageCallback, Request, Response};

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Identities Callback.
#[allow(clippy::module_name_repetitions)]
pub type IdentitiesCallback = MessageCallback<IdentitiesResponse>;

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Identities Request.
#[allow(clippy::module_name_repetitions)]
pub type IdentitiesRequest = Message<IdentitiesRequestPayload>;

impl Request<IdentitiesRequestPayload, IdentitiesResponse> for IdentitiesRequest {
    const ENDPOINT: &'static str = "/request/identities";
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

impl Response<IdentitiesResponsePayload> for IdentitiesResponse {
    const ENDPOINT: &'static str = "/response/identities";
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
