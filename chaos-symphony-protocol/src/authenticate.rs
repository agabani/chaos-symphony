use serde::{Deserialize, Serialize};

use crate::{Identity, Message, MessageCallback, Request, Response};

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Authenticate Callback.
#[allow(clippy::module_name_repetitions)]
pub type AuthenticateCallback = MessageCallback<AuthenticateResponse>;

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Authenticate Request.
#[allow(clippy::module_name_repetitions)]
pub type AuthenticateRequest = Message<AuthenticateRequestPayload>;

impl Request<AuthenticateRequestPayload, AuthenticateResponse> for AuthenticateRequest {
    const ENDPOINT: &'static str = "/request/authenticate";
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

impl Response<AuthenticateResponsePayload> for AuthenticateResponse {
    const ENDPOINT: &'static str = "/response/authenticate";
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
