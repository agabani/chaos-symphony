use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{network::NetworkEndpointId, ship::Ship, types::Identity};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ShipEvent, ShipEventPayload};

use crate::types::Replicate;

/// Ship Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, replicate);
    }
}

fn replicate(
    mut commands: Commands,
    replicates: Query<(Entity, &NetworkEndpointId, &Replicate<Ship>)>,
    endpoints: Query<&NetworkEndpoint>,
    identities: Query<&Identity, With<Ship>>,
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

        let Some(identity) = identities.iter().find(|i| **i == replicate.identity) else {
            warn!("identity not found");
            return;
        };

        let message = ShipEvent::new(
            Uuid::new_v4(),
            ShipEventPayload {
                identity: identity.clone().into(),
            },
        );

        if message.try_send(endpoint).is_err() {
            warn!("failed to send response");
        }

        info!("response sent");
    });
}
