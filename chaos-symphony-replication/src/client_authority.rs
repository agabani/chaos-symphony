use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{network::NetworkEndpointId, types::ClientAuthority, types::Identity};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ClientAuthorityEvent, ClientAuthorityEventPayload};

use crate::types::Replicate;

/// Client Authority Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ClientAuthorityPlugin;

impl Plugin for ClientAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, replicate);
    }
}

fn replicate(
    mut commands: Commands,
    replicates: Query<(Entity, &NetworkEndpointId, &Replicate<ClientAuthority>)>,
    endpoints: Query<&NetworkEndpoint>,
    identities: Query<(&Identity, &ClientAuthority)>,
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

        let Some((identity, client_authority)) =
            identities.iter().find(|(i, _)| **i == replicate.identity)
        else {
            warn!("identity not found");
            return;
        };

        let message = ClientAuthorityEvent::new(
            Uuid::new_v4(),
            ClientAuthorityEventPayload {
                identity: identity.clone().into(),
                authority: client_authority.identity().clone().into(),
            },
        );

        if message.try_send(endpoint).is_err() {
            warn!("failed to send response");
        }

        info!("response sent");
    });
}
