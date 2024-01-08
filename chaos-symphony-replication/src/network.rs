use bevy::prelude::*;
use chaos_symphony_ecs::{
    network::{dispatch, NetworkEndpointId, NetworkMessage},
    types::NetworkIdentity,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, EntityIdentitiesRequest, EntityIdentityEvent, Event as _, PingEvent,
    ReplicateEntityComponentsRequest, Request as _, TransformationEvent,
};

/// Route.
pub fn route(
    commands: &mut Commands,
    endpoint: &NetworkEndpoint,
    identity: Option<&NetworkIdentity>,
    message: chaos_symphony_network::Message,
) -> Option<chaos_symphony_network::Message> {
    match message.endpoint.as_str() {
        AuthenticateRequest::ENDPOINT => {
            dispatch(
                commands,
                endpoint,
                identity,
                AuthenticateRequest::from(message),
            );
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
            dispatch(
                commands,
                endpoint,
                identity,
                ReplicateEntityComponentsRequest::from(message),
            );
            None
        }
        TransformationEvent::ENDPOINT => {
            dispatch(
                commands,
                endpoint,
                identity,
                TransformationEvent::from(message),
            );
            None
        }
        _ => Some(message),
    }
}
