use bevy::prelude::*;
use chaos_symphony_network::Server;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkRecv};
use chaos_symphony_protocol::{Event, TransformationEvent};

use crate::types::NetworkIdentity;

/// Replication Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicationPlugin {
    /// Mode.
    pub mode: ReplicationMode,
}

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, router);

        match self.mode {
            ReplicationMode::Client => {
                app.add_systems(Update, client_send_untrusted_events)
                    .add_systems(Update, client_apply_untrusted_events)
                    .add_systems(Update, client_apply_trusted_events);
            }
            ReplicationMode::Replication => {
                app.add_systems(Update, replication_send_untrusted_events)
                    .add_systems(Update, replication_send_trusted_events)
                    .add_systems(Update, replication_apply_trusted_events);
            }
            ReplicationMode::Simulation => {
                app.add_systems(Update, simulation_send_trusted_events)
                    .add_systems(Update, simulation_apply_trusted_events);

                app.add_systems(Update, simulation_validate_events);
            }
        }
    }
}

/// Replication Mode.
pub enum ReplicationMode {
    /// Client.
    Client,

    /// Replication.
    Replication,

    /// Simulation.
    Simulation,
}

/*
 * ============================================================================
 * Send Events
 * ============================================================================
 */

fn router(mut commands: Commands, _query: Query<(&NetworkEndpoint, Option<&NetworkIdentity>)>) {
    _query.for_each(|(endpoint, identity)| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;

            match message.endpoint.as_str() {
                TransformationEvent::ENDPOINT => {
                    let mut message = TransformationEvent::from(message);

                    if let Some(identity) = identity {
                        match identity.inner.noun.as_str() {
                            "client" | "simulation" => {
                                message.header.source_identity = Some(identity.inner.clone().into())
                            }
                            "replication" => {}
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
                            "simulation" => {
                                commands.add(|world: &mut World| {
                                    world.send_event(Trusted { inner: message });
                                });
                            }
                            noun => todo!("{noun}"),
                        },
                        None => {}
                    }
                }
                _ => {}
            }
        }
    });
}

fn client_send_untrusted_events(mut events: EventReader<Untrusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // send to replication.
    });
}

fn client_apply_untrusted_events(mut events: EventReader<Untrusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // apply.
    });
}

fn client_apply_trusted_events(mut events: EventReader<Trusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // apply.
    });
}

fn simulation_send_trusted_events(mut events: EventReader<Trusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // send to replication.
    });
}

fn simulation_apply_trusted_events(mut events: EventReader<Trusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // send to replication.
    });
}

fn simulation_validate_events(
    mut _reader: EventReader<Untrusted<TransformationEvent>>,
    mut _writer: EventReader<Trusted<TransformationEvent>>,
) {
}

fn replication_send_untrusted_events(mut events: EventReader<Untrusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // send to simulation[authoritative].
    });
}

fn replication_send_trusted_events(mut events: EventReader<Trusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // send to clients+replication+simulation[non-authoritative].
    });
}

fn replication_apply_trusted_events(mut events: EventReader<Trusted<TransformationEvent>>) {
    events.read().for_each(|_event| {
        // apply.
    });
}

#[derive(Event)]
struct Trusted<T> {
    inner: T,
}

#[derive(Event)]
struct Untrusted<T> {
    inner: T,
}
