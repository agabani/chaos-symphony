use bevy::prelude::*;
use chaos_symphony_protocol::TransformationEvent;

use crate::{
    network::NetworkMessage,
    types::{EntityIdentity, Transformation},
};

/// Replication Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicationPlugin {
    /// Mode.
    pub mode: ReplicationMode,
}

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        match self.mode {
            ReplicationMode::Client => {
                app.add_systems(Update, client_send_events)
                    .add_systems(Update, client_recv_events);
            }
            ReplicationMode::Replication => {
                app.add_systems(Update, replication_send_events)
                    .add_systems(Update, replication_recv_events_from_client)
                    .add_systems(Update, replication_recv_events_from_simulation);
            }
            ReplicationMode::Simulation => {
                app.add_systems(Update, simulation_send_events)
                    .add_systems(Update, simulation_recv_events);
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
fn client_send_events(query: Query<(&EntityIdentity, &Transformation), Changed<Transformation>>) {
    query.for_each(|(_identity, _component)| {
        // send to replication.
    });
}

fn simulation_send_events(
    query: Query<(&EntityIdentity, &Transformation), Changed<Transformation>>,
) {
    query.for_each(|(_identity, _component)| {
        // send to replication.
    });
}

fn replication_send_events(
    query: Query<(&EntityIdentity, &Transformation), Changed<Transformation>>,
) {
    query.for_each(|(_identity, _component)| {
        // send to clients+replication+simulation.
    });
}

/*
 * ============================================================================
 * Recv Events
 * ============================================================================
 */
fn client_recv_events(query: Query<&NetworkMessage<TransformationEvent>, With<FromReplication>>) {
    query.for_each(|_message| {
        // apply
    });
}

fn replication_recv_events_from_client(
    query: Query<&NetworkMessage<TransformationEvent>, With<FromClient>>,
) {
    query.for_each(|_message| {
        // forward to simulation
    });
}

fn simulation_recv_events(
    query: Query<&NetworkMessage<TransformationEvent>, With<FromReplication>>,
) {
    query.for_each(|_message| {
        // validate
        //   apply
    });
}

fn replication_recv_events_from_simulation(
    query: Query<&NetworkMessage<TransformationEvent>, With<FromSimulation>>,
) {
    query.for_each(|_message| {
        // apply
    });
}

#[derive(Component)]
struct FromClient;

#[derive(Component)]
struct FromReplication;

#[derive(Component)]
struct FromSimulation;
