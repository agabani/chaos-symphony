use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    network::{NetworkEndpointId, NetworkMessage},
    types::Identity,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    IdentitiesEvent, IdentitiesEventPayload, IdentitiesRequest, IdentitiesResponse,
    IdentitiesResponsePayload,
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
    client_endpoints: Query<&NetworkEndpoint, With<ClientAuthority>>,
    server_endpoints: Query<&NetworkEndpoint, With<ServerAuthority>>,
    identities: Query<&Identity>,
) {
    messages.for_each(|(entity, endpoint_id, message)| {
        let span = error_span!("request", message_id =% message.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let message = &message.inner;

        let Some(endpoint) = server_endpoints
            .iter()
            .chain(client_endpoints.iter())
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
            let message = IdentitiesEvent::new(
                Uuid::new_v4(),
                IdentitiesEventPayload {
                    identity: identity.clone().into(),
                },
            );
            if message.try_send(endpoint).is_err() {
                warn!("failed to send event");
            }
        });
    });
}
