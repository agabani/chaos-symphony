use bevy::prelude::*;
use chaos_symphony_network::Message;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ClientAuthorityEvent, IdentityEvent, ReplicateRequest, ServerAuthorityEvent, ShipEvent,
    TransformationEvent,
};

use crate::types::Identity;

/// Network Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NetworkClientAuthority>()
            .register_type::<NetworkEndpointId>()
            .register_type::<NetworkIdentity>()
            .register_type::<NetworkServerAuthority>();
    }
}

/// Network Client Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct NetworkClientAuthority;

/// Network Endpoint Id.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct NetworkEndpointId {
    /// Inner.
    pub inner: usize,
}

/// Network Endpoint Id.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component, Resource, Reflect)]
pub struct NetworkIdentity {
    /// Inner.
    pub inner: Identity,
}

/// Network Message.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Component)]
pub struct NetworkMessage<T> {
    /// Inner.
    pub inner: T,
}

/// Network Server Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct NetworkServerAuthority;

/// Route.
pub fn route(
    commands: &mut Commands,
    endpoint: &NetworkEndpoint,
    message: Message,
) -> Option<Message> {
    match message.endpoint.as_str() {
        ClientAuthorityEvent::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: ClientAuthorityEvent::from(message),
                },
            ));
            None
        }
        IdentityEvent::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: IdentityEvent::from(message),
                },
            ));
            None
        }
        ReplicateRequest::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: ReplicateRequest::from(message),
                },
            ));
            None
        }
        ServerAuthorityEvent::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: ServerAuthorityEvent::from(message),
                },
            ));
            None
        }
        ShipEvent::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: ShipEvent::from(message),
                },
            ));
            None
        }
        TransformationEvent::ENDPOINT => {
            commands.spawn((
                NetworkEndpointId {
                    inner: endpoint.id(),
                },
                NetworkMessage {
                    inner: TransformationEvent::from(message),
                },
            ));
            None
        }
        _ => Some(message),
    }
}
