use bevy::prelude::*;

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

fn router(mut commands: Commands) {
    commands.add(|world: &mut World| {
        world.send_event(UntrustedTransformationEvent {});
    });
}

fn client_send_untrusted_events(mut events: EventReader<UntrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // send to replication.
    });
}

fn client_apply_untrusted_events(mut events: EventReader<UntrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // apply.
    });
}

fn client_apply_trusted_events(mut events: EventReader<TrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // apply.
    });
}

fn simulation_send_trusted_events(mut events: EventReader<TrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // send to replication.
    });
}

fn simulation_apply_trusted_events(mut events: EventReader<TrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // send to replication.
    });
}

fn simulation_validate_events(
    mut _reader: EventReader<UntrustedTransformationEvent>,
    mut _writer: EventReader<TrustedTransformationEvent>,
) {
}

fn replication_send_untrusted_events(mut events: EventReader<UntrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // send to simulation[authoritative].
    });
}

fn replication_send_trusted_events(mut events: EventReader<TrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // send to clients+replication+simulation[non-authoritative].
    });
}

fn replication_apply_trusted_events(mut events: EventReader<TrustedTransformationEvent>) {
    events.read().for_each(|_event| {
        // apply.
    });
}

#[derive(Event)]
struct UntrustedTransformationEvent {}

#[derive(Event)]
struct TrustedTransformationEvent {}
