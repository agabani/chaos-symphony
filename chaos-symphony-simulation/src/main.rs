#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

use std::str::FromStr as _;

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    types::{
        EntityClientAuthority, EntityIdentity, EntityReplicationAuthority,
        EntitySimulationAuthority, Identity, NetworkIdentity, ReplicateSource, Role, Trusted,
    },
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    EntityClientAuthorityEvent, EntityReplicationAuthorityEvent, EntitySimulationAuthorityEvent,
    Event as _, TransformationEvent, TransformationEventPayload,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(chaos_symphony_ecs::DefaultPlugins {
        bevy_config: BevyConfigPlugin {
            headless: false,
            log_filter: "chaos_symphony_simulation".to_string(),
            title: "Chaos Symphony Simulation".to_string(),
        },
        network_identity: NetworkIdentity {
            inner: Identity {
                id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                noun: "simulation".to_string(),
            },
        },
        role: Role::Simulation,
    })
    .add_systems(
        Update,
        (
            test_spawn_entity_identity_after_network_authenticate,
            test_translate_entity_identity_periodically,
        ),
    );

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn test_spawn_entity_identity_after_network_authenticate(
    mut commands: Commands,
    query: Query<(), (With<NetworkEndpoint>, Added<NetworkIdentity>)>,
) {
    query.for_each(|()| {
        commands.spawn((
            EntityIdentity {
                inner: Identity {
                    id: Uuid::new_v4(),
                    noun: "test_simulation".to_string(),
                },
            },
            ReplicateSource,
            PeriodicTimer {
                inner: Timer::from_seconds(1.0, TimerMode::Repeating),
            },
            EntityClientAuthority {
                identity: Identity {
                    id: Uuid::from_str("d908808f-073d-4c57-9c08-bf91ba2b1bce").unwrap(),
                    noun: "ai".to_string(),
                },
            },
            EntityReplicationAuthority {
                identity: Identity {
                    id: Uuid::from_str("84988f7d-2146-4677-b4f8-6d503f72fea3").unwrap(),
                    noun: "replication".to_string(),
                },
            },
            EntitySimulationAuthority {
                identity: Identity {
                    id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                    noun: "simulation".to_string(),
                },
            },
        ));
    });
}

#[derive(Component)]
struct PeriodicTimer {
    inner: Timer,
}

#[allow(clippy::needless_pass_by_value)]
fn test_translate_entity_identity_periodically(
    time: Res<Time>,
    mut query: Query<(&EntityIdentity, &mut PeriodicTimer)>,
    mut writer: EventWriter<Trusted<TransformationEvent>>,
    mut writer_s: EventWriter<Trusted<EntitySimulationAuthorityEvent>>,
    mut writer_r: EventWriter<Trusted<EntityReplicationAuthorityEvent>>,
    mut writer_c: EventWriter<Trusted<EntityClientAuthorityEvent>>,
) {
    query.for_each_mut(|(entity_identity, mut timer)| {
        if timer.inner.tick(time.delta()).finished() {
            let mut message = TransformationEvent::message(
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

            message.header.source_identity = Some(chaos_symphony_protocol::Identity {
                id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                noun: "simulation".to_string(),
            });

            writer.send(Trusted { inner: message });

            {
                let mut message = EntitySimulationAuthorityEvent::message(
                    Uuid::new_v4(),
                    chaos_symphony_protocol::EntitySimulationAuthorityEventPayload {
                        authority_identity: Identity {
                            id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                            noun: "simulation".to_string(),
                        }
                        .into(),
                        entity_identity: entity_identity.inner.clone().into(),
                    },
                );

                message.header.source_identity = Some(chaos_symphony_protocol::Identity {
                    id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                    noun: "simulation".to_string(),
                });

                writer_s.send(Trusted { inner: message });
            }

            {
                let mut message = EntityReplicationAuthorityEvent::message(
                    Uuid::new_v4(),
                    chaos_symphony_protocol::EntityReplicationAuthorityEventPayload {
                        authority_identity: Identity {
                            id: Uuid::from_str("84988f7d-2146-4677-b4f8-6d503f72fea3").unwrap(),
                            noun: "replication".to_string(),
                        }
                        .into(),
                        entity_identity: entity_identity.inner.clone().into(),
                    },
                );

                message.header.source_identity = Some(chaos_symphony_protocol::Identity {
                    id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                    noun: "simulation".to_string(),
                });

                writer_r.send(Trusted { inner: message });
            }

            {
                let mut message = EntityClientAuthorityEvent::message(
                    Uuid::new_v4(),
                    chaos_symphony_protocol::EntityClientAuthorityEventPayload {
                        authority_identity: Identity {
                            id: Uuid::from_str("d908808f-073d-4c57-9c08-bf91ba2b1bce").unwrap(),
                            noun: "ai".to_string(),
                        }
                        .into(),
                        entity_identity: entity_identity.inner.clone().into(),
                    },
                );

                message.header.source_identity = Some(chaos_symphony_protocol::Identity {
                    id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                    noun: "simulation".to_string(),
                });

                writer_c.send(Trusted { inner: message });
            }
        }
    });
}
