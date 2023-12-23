use bevy::prelude::*;
use chaos_symphony_network::Message;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::IdentitiesEvent;

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
pub fn route(
    commands: &mut Commands,
    endpoint: &NetworkEndpoint,
    message: Message,
) -> Option<Message> {
    match message.endpoint.as_str() {
        IdentitiesEvent::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: IdentitiesEvent::from(message),
                },
            ));
            None
        }
        _ => Some(message),
    }
}
