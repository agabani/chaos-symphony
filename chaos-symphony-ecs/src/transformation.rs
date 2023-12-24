use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{Event as _, TransformationEvent};

use crate::{
    network::NetworkMessage,
    types::{EntityIdentity, NetworkIdentity, ReplicateEntity, Transformation},
};

/// Transformation Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct TransformationPlugin;

impl Plugin for TransformationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (event, replicate));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn event(
    mut commands: Commands,
    messages: Query<(Entity, &NetworkMessage<TransformationEvent>)>,
    entity_identities: Query<(Entity, &EntityIdentity)>,
) {
    messages.for_each(|(entity, message)| {
        let message = &message.inner;
        commands.entity(entity).despawn();

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let payload = &message.payload;

        let entity_identity = EntityIdentity {
            inner: payload.entity_identity.clone().into(),
        };
        let Some((entity, _)) = entity_identities
            .iter()
            .find(|(_, i)| **i == entity_identity)
        else {
            warn!("entity does not exist");
            return;
        };

        let transformation: Transformation = payload.transformation.into();
        commands.entity(entity).insert(transformation);
        info!(entity_identity =? entity_identity, "updated");
    });
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
