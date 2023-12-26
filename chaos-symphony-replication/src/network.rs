use bevy::prelude::*;
use chaos_symphony_ecs::network::{NetworkEndpointId, NetworkMessage};
use chaos_symphony_network::Message;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, EntityIdentitiesRequest, EntityIdentityEvent, Event as _, PingEvent,
    ReplicateEntityComponentsRequest, Request as _,
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
        EntityIdentitiesRequest::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: EntityIdentitiesRequest::from(message),
                },
            ));
            None
        }
        EntityIdentityEvent::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: EntityIdentityEvent::from(message),
                },
            ));
            None
        }
        PingEvent::ENDPOINT => None,
        ReplicateEntityComponentsRequest::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: ReplicateEntityComponentsRequest::from(message),
                },
            ));
            None
        }
        _ => Some(message),
    }
}
