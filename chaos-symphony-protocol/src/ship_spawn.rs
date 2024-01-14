use serde::{Deserialize, Serialize};

use crate::{Identity, Message, MessageCallback, Request, Response};

/*
 * ============================================================================
 * Callback
 * ============================================================================
 */

/// Ship Spawn Callback.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnCallback = MessageCallback<ShipSpawnResponse>;

/*
 * ============================================================================
 * Request
 * ============================================================================
 */

/// Ship Spawn Request.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnRequest = Message<ShipSpawnRequestPayload>;

impl Request<ShipSpawnRequestPayload, ShipSpawnResponse> for ShipSpawnRequest {
    const ENDPOINT: &'static str = "/request/ship_spawn";
}

/// Ship Spawn Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipSpawnRequestPayload {
    /// Entity Identity.
    pub entity_identity: Identity,
}

/*
 * ============================================================================
 * Response
 * ============================================================================
 */

/// Ship Spawn Response.
#[allow(clippy::module_name_repetitions)]
pub type ShipSpawnResponse = Message<ShipSpawnResponsePayload>;

impl Response<ShipSpawnResponsePayload> for ShipSpawnResponse {
    const ENDPOINT: &'static str = "/response/ship_spawn";
}

/// Ship Spawn Response Payload.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ShipSpawnResponsePayload {
    /// Failure.
    Failure,

    /// Success.
    Success,
}
