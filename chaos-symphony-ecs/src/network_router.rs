use std::fmt::Debug;

use bevy::prelude::*;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkRecv};
use chaos_symphony_protocol::{
    AuthenticateRequest, EntityClientAuthorityEvent, EntityIdentitiesRequest, EntityIdentityEvent,
    EntityReplicationAuthorityEvent, EntitySimulationAuthorityEvent, Event, Message, PingEvent,
    ReplicateEntityComponentsRequest, Request as _, ShipEvent, TransformationEvent,
};

use crate::types::{NetworkIdentity, Trusted, Untrusted};

/// Network Router.
pub struct NetworkRouter;

impl Plugin for NetworkRouter {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, route);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn route(mut commands: Commands, endpoints: Query<(&NetworkEndpoint, Option<&NetworkIdentity>)>) {
    endpoints.for_each(|(endpoint, identity)| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;
            match message.endpoint.as_str() {
                AuthenticateRequest::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        AuthenticateRequest::from(message),
                    );
                }
                EntityClientAuthorityEvent::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        EntityClientAuthorityEvent::from(message),
                    );
                }
                EntityReplicationAuthorityEvent::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        EntityReplicationAuthorityEvent::from(message),
                    );
                }
                EntitySimulationAuthorityEvent::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        EntitySimulationAuthorityEvent::from(message),
                    );
                }
                EntityIdentitiesRequest::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        EntityIdentitiesRequest::from(message),
                    );
                }
                EntityIdentityEvent::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        EntityIdentityEvent::from(message),
                    );
                }
                PingEvent::ENDPOINT => {}
                ReplicateEntityComponentsRequest::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        ReplicateEntityComponentsRequest::from(message),
                    );
                }
                ShipEvent::ENDPOINT => {
                    dispatch(
                        //
                        &mut commands,
                        endpoint,
                        identity,
                        ShipEvent::from(message),
                    );
                }
                TransformationEvent::ENDPOINT => {
                    dispatch(
                        &mut commands,
                        endpoint,
                        identity,
                        TransformationEvent::from(message),
                    );
                }
                endpoint => warn!(endpoint, "unhandled"),
            }
        }
    });
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
    } else {
        message.header.source_identity = None;
    }

    match &message.header.source_identity {
        Some(identity) => match identity.noun.as_str() {
            "ai" | "client" => {
                commands.add(|world: &mut World| {
                    let event = Untrusted { inner: message };
                    // warn!("{event:?}");
                    world.send_event(event);
                });
            }
            "replication" | "simulation" => {
                commands.add(|world: &mut World| {
                    let event = Trusted { inner: message };
                    // warn!("{event:?}");
                    world.send_event(event);
                });
            }
            noun => todo!("{noun}"),
        },
        None => {
            commands.add(|world: &mut World| {
                let event = Untrusted { inner: message };
                // warn!("{event:?}");
                world.send_event(event);
            });
        }
    }
}
