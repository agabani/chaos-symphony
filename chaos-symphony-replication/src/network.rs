use std::fmt::Debug;

use bevy::prelude::*;
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    types::{NetworkIdentity, Trusted, Untrusted},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, EntityIdentitiesRequest, EntityIdentityEvent, Event as _, Message,
    PingEvent, ReplicateEntityComponentsRequest, Request as _, TransformationEvent,
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
            dispatch(
                commands,
                identity,
                ReplicateEntityComponentsRequest::from(message),
            );
            None
        }
        TransformationEvent::ENDPOINT => {
            dispatch(commands, identity, TransformationEvent::from(message));
            None
        }
        _ => Some(message),
    }
}

fn dispatch<T>(commands: &mut Commands, identity: Option<&NetworkIdentity>, mut message: Message<T>)
where
    T: Send + Sync + 'static + Debug,
{
    if let Some(identity) = identity {
        match identity.inner.noun.as_str() {
            "client" | "simulation" => {
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
            "client" => {
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
