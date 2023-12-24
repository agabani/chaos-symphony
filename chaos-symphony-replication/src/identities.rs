use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkIdentity, NetworkMessage},
    types::Identity,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    IdentitiesRequest, IdentitiesResponse, IdentitiesResponsePayload, IdentityEvent,
    IdentityEventPayload,
};

/// Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct IdentitiesPlugin;

impl Plugin for IdentitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, request);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut commands: Commands,
    messages: Query<(
        Entity,
        &NetworkEndpointId,
        &NetworkMessage<IdentitiesRequest>,
    )>,
    endpoints: Query<&NetworkEndpoint, With<NetworkIdentity>>,
    identities: Query<&Identity>,
) {
    messages.for_each(|(entity, endpoint_id, message)| {
        let span = error_span!("request", message_id =% message.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let message = &message.inner;

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == endpoint_id.inner)
        else {
            warn!("endpoint not found");
            return;
        };

        let response = IdentitiesResponse::new(message.id, IdentitiesResponsePayload::Success);
        if response.try_send(endpoint).is_err() {
            warn!("failed to send response");
            return;
        }

        identities.for_each(|identity| {
            let message = IdentityEvent::new(
                Uuid::new_v4(),
                IdentityEventPayload {
                    identity: identity.clone().into(),
                },
            );
            if message.try_send(endpoint).is_err() {
                warn!("failed to send event");
            }
        });
    });
}
