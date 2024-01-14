use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{EntityIdentityEvent, EntityIdentityEventPayload, Event};

use crate::types::{
    EntityIdentity, EntityReplicationAuthority, EntitySimulationAuthority, NetworkIdentity,
    ReplicateSource, Role, Trusted,
};

/// Entity Identity Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct EntityIdentityPlugin {
    role: Role,
}

impl EntityIdentityPlugin {
    /// Creates a new [`EntityIdentityPlugin`].
    #[must_use]
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

impl Plugin for EntityIdentityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Trusted<EntityIdentityEvent>>();

        app.add_systems(Update, apply_trusted_event);

        match self.role {
            Role::Client => {}
            Role::Replication => {
                app.add_systems(Update, send_trusted_event);
            }
            Role::Simulation => {
                app.add_systems(Update, broadcast_on_change);
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

        let mut entity = commands.spawn(EntityIdentity {
            inner: trusted.inner.payload.inner.clone().into(),
        });

        /*
         * ReplicateEntityComponentsPlugin will need to replicate components,
         * which requires knowledge of where the components originated from.
         */
        if let Some(network_identity) = &trusted.inner.header.source_identity {
            match network_identity.noun.as_str() {
                "replication" => {
                    entity.insert(EntityReplicationAuthority {
                        identity: network_identity.clone().into(),
                    });
                }
                "simulation" => {
                    entity.insert(EntitySimulationAuthority {
                        identity: network_identity.clone().into(),
                    });
                }
                noun => {
                    todo!("{noun}");
                }
            }
        }
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
fn broadcast_on_change(
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
