#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

mod entity_identities;
mod entity_identity;
mod network;
mod network_authenticate;
mod replicate_entity_components;

use std::{str::FromStr as _, sync::mpsc::TryRecvError};

use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    network_authority::NetworkAuthorityPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    transformation::TransformationPlugin,
    types::{EntityIdentity, Identity, NetworkIdentity, Transformation},
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};
use chaos_symphony_protocol::{Event, TransformationEvent, TransformationEventPayload};
use entity_identities::EntityIdentitiesPlugin;
use entity_identity::EntityIdentityPlugin;
use network_authenticate::NetworkAuthenticatePlugin;
use replicate_entity_components::ReplicateEntityComponentsPlugin;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(BevyConfigPlugin {
        headless: false,
        log_filter: "chaos_symphony_replication".to_string(),
        title: "Chaos Symphony Replication".to_string(),
    })
    // Default Plugins (Network)
    .add_plugins((
        NetworkPlugin {
            client: false,
            server: true,
        },
        NetworkAuthenticatePlugin {
            identity: NetworkIdentity {
                inner: Identity {
                    id: Uuid::from_str("84988f7d-2146-4677-b4f8-6d503f72fea3").unwrap(),
                    noun: "replication".to_string(),
                },
            },
        },
        NetworkAuthorityPlugin,
        NetworkDisconnectPlugin,
    ))
    // Default Plugins
    .add_plugins((
        EntityIdentitiesPlugin,
        EntityIdentityPlugin,
        ReplicateEntityComponentsPlugin,
        TransformationPlugin,
    ))
    // ...
    .add_systems(Update, (accepted, route))
    .add_systems(Startup, testing)
    .add_systems(Update, testing_events);

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn accepted(mut commands: Commands, server: Res<NetworkServer>) {
    loop {
        match server.try_recv() {
            Ok(endpoint) => {
                let id = endpoint.id();
                let remote_address = endpoint.remote_address();

                let entity = commands.spawn(endpoint).id();

                let span =
                    info_span!("accept", entity =? entity, id, remote_address =% remote_address);
                let _guard = span.enter();
                info!("connected");
            }
            Err(TryRecvError::Disconnected) => {
                panic!("[network:server] disconnected");
            }
            Err(TryRecvError::Empty) => {
                return;
            }
        };
    }
}

#[allow(clippy::match_single_binding)]
#[allow(clippy::needless_pass_by_value)]
fn route(mut commands: Commands, endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;
            if let Some(message) = network::route(&mut commands, endpoint, message) {
                match message.endpoint.as_str() {
                    endpoint => {
                        warn!(endpoint, "unhandled");
                    }
                }
            }
        }
    });
}

fn testing(mut commands: Commands) {
    commands.spawn((
        EntityIdentity {
            inner: Identity {
                id: Uuid::new_v4(),
                noun: "test_replication".to_string(),
            },
        },
        Transformation {
            orientation: DQuat::from_rotation_z(0.0),
            position: DVec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
        RandomTimer {
            inner: Timer::from_seconds(1.0, TimerMode::Repeating),
        },
    ));
}

#[derive(Component)]
struct RandomTimer {
    inner: Timer,
}

#[allow(clippy::needless_pass_by_value)]
fn testing_events(
    time: Res<Time>,
    mut query: Query<(&EntityIdentity, &mut RandomTimer)>,
    endpoints: Query<&NetworkEndpoint, With<NetworkIdentity>>,
) {
    query.for_each_mut(|(entity_identity, mut timer)| {
        if timer.inner.tick(time.delta()).finished() {
            endpoints.for_each(|endpoint| {
                let message = TransformationEvent::message(
                    Uuid::new_v4(),
                    TransformationEventPayload {
                        entity_identity: entity_identity.inner.clone().into(),
                        transformation: chaos_symphony_protocol::Transformation {
                            orientation: chaos_symphony_protocol::Orientation {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                                w: 1.0,
                            },
                            position: chaos_symphony_protocol::Position {
                                x: time.elapsed_seconds_f64(),
                                y: time.elapsed_seconds_f64(),
                                z: time.elapsed_seconds_f64(),
                            },
                        },
                    },
                );

                if message.try_send(endpoint).is_err() {
                    error!("failed to send test events");
                };
            });
        }
    });
}
