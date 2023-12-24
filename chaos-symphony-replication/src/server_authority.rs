use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    network::NetworkEndpointId,
    types::{EntityServerAuthority, Identity},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ServerAuthorityEvent, ServerAuthorityEventPayload};

use crate::types::Replicate;

/// Server Authority Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ServerAuthorityPlugin;

impl Plugin for ServerAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, replicate);
    }
}

fn replicate(
    mut commands: Commands,
    replicates: Query<(
        Entity,
        &NetworkEndpointId,
        &Replicate<EntityServerAuthority>,
    )>,
    endpoints: Query<&NetworkEndpoint>,
    identities: Query<(&Identity, &EntityServerAuthority)>,
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

        let Some((identity, server_authority)) =
            identities.iter().find(|(i, _)| **i == replicate.identity)
        else {
            warn!("identity not found");
            return;
        };

        let message = ServerAuthorityEvent::new(
            Uuid::new_v4(),
            ServerAuthorityEventPayload {
                identity: identity.clone().into(),
                authority: server_authority.identity().clone().into(),
            },
        );

        if message.try_send(endpoint).is_err() {
            warn!("failed to send response");
        }

        info!("response sent");
    });
}
