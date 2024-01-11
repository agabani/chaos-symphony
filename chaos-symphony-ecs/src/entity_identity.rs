use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{EntityIdentityEvent, EntityIdentityEventPayload, Event};

use crate::{
    replication::ReplicationMode,
    types::{EntityIdentity, NetworkIdentity, ReplicateSource, Trusted},
};

/// Entity Identity Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct EntityIdentityPlugin {
    mode: ReplicationMode,
}

impl EntityIdentityPlugin {
    /// Creates a new [`EntityIdentityPlugin`].
    #[must_use]
    pub fn new(mode: ReplicationMode) -> Self {
        Self { mode }
    }
}

impl Plugin for EntityIdentityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Trusted<EntityIdentityEvent>>();

        app.add_systems(Update, apply_trusted_event);

        match self.mode {
            ReplicationMode::Client => {}
            ReplicationMode::Replication => {
                app.add_systems(Update, send_trusted_event);
            }
            ReplicationMode::Simulation => {
                app.add_systems(Update, replicate_changed);
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn apply_trusted_event(
    mut commands: Commands,
    mut reader: EventReader<Trusted<EntityIdentityEvent>>,
    query: Query<&EntityIdentity>,
) {
    reader.read().for_each(|trusted| {
        let span = error_span!("event", message_id =%  trusted.inner.id);
        let _guard = span.enter();

        if query
            .iter()
            .any(|entity_identity| entity_identity.inner == trusted.inner.payload.inner)
        {
            warn!("entity does exist");
            return;
        }

        commands.spawn(EntityIdentity {
            inner: trusted.inner.payload.inner.clone().into(),
        });
    });
}

#[allow(clippy::needless_pass_by_value)]
fn send_trusted_event(
    mut reader: EventReader<Trusted<EntityIdentityEvent>>,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity)>,
) {
    reader.read().for_each(|event| {
        endpoints
            .iter()
            .filter(|(_, network_identity)| {
                network_identity.inner != *event.inner.header.source_identity.as_ref().unwrap()
            })
            .for_each(|(endpoint, _)| {
                let message = event.inner.clone();
                if message.try_send(endpoint).is_err() {
                    error!("failed to send event");
                };
            });
    });
}

#[allow(clippy::needless_pass_by_value)]
fn replicate_changed(
    entity_identities: Query<&EntityIdentity, (Changed<EntityIdentity>, With<ReplicateSource>)>,
    endpoints: Query<&NetworkEndpoint, With<NetworkIdentity>>,
) {
    entity_identities.for_each(|entity_identity| {
        endpoints.for_each(|endpoint| {
            let message = EntityIdentityEvent::message(
                Uuid::new_v4(),
                EntityIdentityEventPayload {
                    inner: entity_identity.inner.clone().into(),
                },
            );

            if message.try_send(endpoint).is_err() {
                error!("failed to send message");
            }
        });
    });
}
