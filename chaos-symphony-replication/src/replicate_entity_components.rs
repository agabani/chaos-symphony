use bevy::prelude::*;

use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    types::{EntityIdentity, NetworkIdentity},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ReplicateEntityComponentsRequest, ReplicateEntityComponentsResponse,
    ReplicateEntityComponentsResponsePayload, Response as _,
};

/// Replicate Entity Components Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicateEntityComponentsPlugin;

impl Plugin for ReplicateEntityComponentsPlugin {
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
        &NetworkMessage<ReplicateEntityComponentsRequest>,
    )>,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity)>,
    entity_identities: Query<&EntityIdentity>,
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

        let payload = &request.inner.payload;

        let entity_identity = EntityIdentity {
            inner: payload.entity_identity.clone().into(),
        };

        if !entity_identities.iter().any(|i| *i == entity_identity) {
            let response = ReplicateEntityComponentsResponse::message(
                request.inner.id,
                ReplicateEntityComponentsResponsePayload::Failure,
            );
            if response.try_send(endpoint).is_err() {
                warn!("failed to send response");
            }
            return;
        }

        let response = ReplicateEntityComponentsResponse::message(
            request.inner.id,
            ReplicateEntityComponentsResponsePayload::Success,
        );
        if response.try_send(endpoint).is_err() {
            warn!("failed to send response");
        }
    });
}
