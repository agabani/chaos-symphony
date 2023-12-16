#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Protocol

use chaos_symphony_network::Payload;

/// Authenticate Request
pub struct AuthenticateRequest {
    /// Id.
    pub id: String,

    /// Identity.
    pub identity: String,
}

impl From<Payload> for AuthenticateRequest {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            identity: value.properties.remove("identity").unwrap(),
        }
    }
}

impl From<AuthenticateRequest> for Payload {
    fn from(value: AuthenticateRequest) -> Self {
        Self {
            id: value.id,
            endpoint: "/request/authenticate".to_string(),
            properties: std::collections::HashMap::from([("identity".to_string(), value.identity)]),
        }
    }
}

/// Authenticate Response.
pub struct AuthenticateResponse {
    /// Id.
    pub id: String,

    /// Success.
    pub success: bool,
}

impl From<Payload> for AuthenticateResponse {
    fn from(mut value: Payload) -> Self {
        Self {
            id: value.id,
            success: value.properties.remove("success").unwrap().parse().unwrap(),
        }
    }
}

impl From<AuthenticateResponse> for Payload {
    fn from(value: AuthenticateResponse) -> Self {
        Self {
            id: value.id,
            endpoint: "/response/authenticate".to_string(),
            properties: std::collections::HashMap::from([(
                "success".to_string(),
                value.success.to_string(),
            )]),
        }
    }
}
