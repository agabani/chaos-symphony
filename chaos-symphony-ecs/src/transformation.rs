use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{Event as _, TransformationEvent};

use crate::types::{EntityIdentity, NetworkIdentity, ReplicateEntity, Transformation};

/// Transformation Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct TransformationPlugin;

impl Plugin for TransformationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, replicate);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn replicate(
    mut commands: Commands,
    replicates: Query<(Entity, &NetworkIdentity, &ReplicateEntity<Transformation>)>,
    endpoints: Query<(&NetworkIdentity, &NetworkEndpoint)>,
    entity_identities: Query<(&EntityIdentity, &Transformation)>,
) {
    replicates.for_each(|(entity, network_identity, replicate)| {
        commands.entity(entity).despawn();

        let Some((_, endpoint)) = endpoints.iter().find(|(i, _)| *i == network_identity) else {
            warn!("endpoint does not exist");
            return;
        };

        let Some((entity_identity, transformation)) = entity_identities
            .iter()
            .find(|(i, _)| **i == replicate.identity)
        else {
            warn!("entity does not exist");
            return;
        };

        let event = TransformationEvent::message(
            Uuid::new_v4(),
            chaos_symphony_protocol::TransformationEventPayload {
                entity_identity: entity_identity.inner.clone().into(),
                transformation: (*transformation).into(),
            },
        );

        if event.try_send(endpoint).is_err() {
            warn!("failed to send event");
        }
    });
}
