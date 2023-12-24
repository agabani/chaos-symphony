use bevy::prelude::*;
use chaos_symphony_network_bevy::NetworkEndpoint;

/// Network Endpoint Id.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component)]
pub struct NetworkEndpointId {
    /// Inner.
    pub inner: usize,
}

/// Network Message.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component)]
pub struct NetworkMessage<T> {
    /// Inner.
    pub inner: T,
}

/// Route.
#[allow(clippy::match_single_binding)]
pub fn route(
    _commands: &mut Commands,
    _endpoint: &NetworkEndpoint,
    message: chaos_symphony_network::Message,
) -> Option<chaos_symphony_network::Message> {
    match message.endpoint.as_str() {
        _ => Some(message),
    }
}
