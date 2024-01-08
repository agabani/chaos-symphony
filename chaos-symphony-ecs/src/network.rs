use std::fmt::Debug;

use bevy::prelude::*;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{EntityIdentityEvent, Event as _, Message, TransformationEvent};

use crate::types::{NetworkIdentity, Trusted, Untrusted};

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
    identity: Option<&NetworkIdentity>,
    message: chaos_symphony_network::Message,
) -> Option<chaos_symphony_network::Message> {
    match message.endpoint.as_str() {
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

/// Dispatch.
pub fn dispatch<T>(
    commands: &mut Commands,
    endpoint: &NetworkEndpoint,
    identity: Option<&NetworkIdentity>,
    mut message: Message<T>,
) where
    T: Send + Sync + 'static + Debug,
{
    message.header.source_endpoint_id = Some(endpoint.id());

    if let Some(identity) = identity {
        match identity.inner.noun.as_str() {
            "ai" | "client" | "simulation" => {
                // always overwrite source from untrusted endpoints.
                message.header.source_identity = Some(identity.inner.clone().into());
            }
            "replication" => {
                // populate source from trusted endpoints if not present.
                if message.header.source_identity.is_none() {
                    message.header.source_identity = Some(identity.inner.clone().into());
                }
            }
            noun => todo!("{noun}"),
        };
    }

    match &message.header.source_identity {
        Some(identity) => match identity.noun.as_str() {
            "ai" | "client" => {
                commands.add(|world: &mut World| {
                    world.send_event(Untrusted { inner: message });
                });
            }
            "replication" | "simulation" => {
                commands.add(|world: &mut World| {
                    world.send_event(Trusted { inner: message });
                });
            }
            noun => todo!("{noun}"),
        },
        None => {}
    }
}
