use bevy::prelude::*;
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    types::NetworkIdentity,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    IdentitiesRequest, IdentitiesResponse, IdentitiesResponsePayload, Response,
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
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity)>,
) {
    messages.for_each(|(entity, endpoint_id, request)| {
        let span = error_span!("request", message_id =% request.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some((endpoint, _identity)) = endpoints
            .iter()
            .find(|(endpoint, _)| endpoint.id() == endpoint_id.inner)
        else {
            warn!("endpoint not found");
            return;
        };

        let response =
            IdentitiesResponse::message(request.inner.id, IdentitiesResponsePayload::Success);

        if response.try_send(endpoint).is_err() {
            warn!("failed to send response");
        }

        info!("sent response");
    });
}