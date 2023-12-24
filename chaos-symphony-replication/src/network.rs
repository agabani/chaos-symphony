use bevy::prelude::*;
use chaos_symphony_ecs::network::{NetworkEndpointId, NetworkMessage};
use chaos_symphony_network::Message;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, Event as _, IdentitiesRequest, PingEvent, Request as _,
};

/// Route.
pub fn route(
    commands: &mut Commands,
    endpoint: &NetworkEndpoint,
    message: Message,
) -> Option<Message> {
    match message.endpoint.as_str() {
        AuthenticateRequest::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: AuthenticateRequest::from(message),
                },
            ));
            None
        }
        IdentitiesRequest::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: IdentitiesRequest::from(message),
                },
            ));
            None
        }
        PingEvent::ENDPOINT => None,
        _ => Some(message),
    }
}
