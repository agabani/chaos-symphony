use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    ship::{Ship, ShipBundle},
    transform::Transformation,
    types::{ClientAuthority, Identity, ServerAuthority},
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

        let Some(client_authority) = &message.inner.payload.client_authority else {
            error!("client authority is missing");
            let response =
                ShipSpawnResponse::new(message.inner.id, ShipSpawnResponsePayload::Failure);
            if response.try_send(endpoint).is_err() {
                error!("failed to send response to endpoint");
            }
            return;
        };

        let Some(server_authority) = &message.inner.payload.server_authority else {
            error!("server authority is missing");
            let response =
                ShipSpawnResponse::new(message.inner.id, ShipSpawnResponsePayload::Failure);
            if response.try_send(endpoint).is_err() {
                error!("failed to send response to endpoint");
            }
            return;
        };

        let bundle = ShipBundle {
            ship: Ship,
            identity: Identity::new("ship".to_string(), Uuid::new_v4()),
            client_authority: ClientAuthority::new(client_authority.clone().into()),
            server_authority: ServerAuthority::new(server_authority.clone().into()),
            transformation: Transformation {
                orientation: DQuat::from_rotation_z(0.0),
                position: DVec3::ZERO,
            },
        };

        let response = ShipSpawnResponse::new(
            message.inner.id,
            ShipSpawnResponsePayload::Success(ShipSpawnResponsePayloadSuccess {
                identity: bundle.identity.clone().into(),
                client_authority: bundle.client_authority.identity().clone().into(),
                server_authority: bundle.server_authority.identity().clone().into(),
                transformation: bundle.transformation.into(),
            }),
        );

        if response.try_send(endpoint).is_err() {
            error!("failed to send response to endpoint");
            return;
        }

        info!(identity =? bundle.identity.id(), "spawned");
        commands.spawn(bundle);
    });
}
