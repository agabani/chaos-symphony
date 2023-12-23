use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    ship::Ship,
    transform::Transformation,
    types::Identity,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ShipSpawnRequest, ShipSpawnResponse, ShipSpawnResponsePayload, ShipSpawnResponsePayloadSuccess,
};
use tracing::instrument;

#[instrument(skip_all)]
pub fn request(
    mut commands: Commands,
    messages: Query<(
        Entity,
        &NetworkEndpointId,
        &NetworkMessage<ShipSpawnRequest>,
    )>,
    endpoints: Query<&NetworkEndpoint>,
) {
    messages.for_each(|(entity, endpoint_id, message)| {
        let span = error_span!("request", message_id =% message.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == endpoint_id.inner)
        else {
            error!("endpoint not found");
            return;
        };

        let message = &message.inner;

        let identity = Identity::new("ship".to_string(), Uuid::new_v4());

        let response = ShipSpawnResponse::new(
            message.id,
            ShipSpawnResponsePayload::Success(ShipSpawnResponsePayloadSuccess {
                identity: identity.clone().into(),
            }),
        );

        if response.try_send(endpoint).is_err() {
            error!("failed to send response to endpoint");
        }

        info!(identity =? identity.id(), "spawned");
        commands.spawn((
            identity,
            Ship,
            Transformation {
                orientation: DQuat::from_rotation_z(0.0),
                position: DVec3::ZERO,
            },
        ));
    });
}
