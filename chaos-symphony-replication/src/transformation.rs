use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{network::NetworkEndpointId, transform::Transformation, types::Identity};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{TransformationEvent, TransformationEventPayload};

use crate::types::Replicate;

/// Transformation Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct TransformationPlugin;

impl Plugin for TransformationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, replicate);
    }
}

fn replicate(
    mut commands: Commands,
    replicates: Query<(Entity, &NetworkEndpointId, &Replicate<Transformation>)>,
    endpoints: Query<&NetworkEndpoint>,
    identities: Query<(&Identity, &Transformation)>,
) {
    replicates.for_each(|(entity, endpoint_id, replicate)| {
        commands.entity(entity).despawn();

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == endpoint_id.inner)
        else {
            warn!("endpoint not found");
            return;
        };

        let Some((identity, transformation)) =
            identities.iter().find(|(i, _)| **i == replicate.identity)
        else {
            warn!("identity not found");
            return;
        };

        let message = TransformationEvent::new(
            Uuid::new_v4(),
            TransformationEventPayload {
                identity: identity.clone().into(),
                transformation: transformation.clone().into(),
            },
        );

        if message.try_send(endpoint).is_err() {
            warn!("failed to send response");
        }

        info!("response sent");
    });
}
