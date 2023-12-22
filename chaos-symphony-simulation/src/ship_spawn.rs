use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    routing::{EndpointId, Request},
    ship::{Ship, ShipBundle},
    transform::Transformation,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ShipSpawnRequest, ShipSpawnResponse, ShipSpawnResponsePayload};
use tracing::instrument;

#[instrument(skip_all)]
pub fn request(
    mut commands: Commands,
    requests: Query<(Entity, &EndpointId, &Request<ShipSpawnRequest>)>,
    endpoints: Query<&NetworkEndpoint>,
) {
    requests.for_each(|(entity, endpoint_id, request)| {
        let span = error_span!("request", request_id = request.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == endpoint_id.inner)
        else {
            error!("endpoint not found");
            return;
        };

        let request = &request.inner;

        let bundle = ShipBundle {
            ship: Ship,
            identity: Identity::new(Uuid::new_v4().to_string()),
            client_authority: ClientAuthority::new(request.payload.client_authority.clone()),
            server_authority: ServerAuthority::new(request.payload.server_authority.clone()),
            transformation: Transformation {
                orientation: DQuat::from_rotation_z(0.0),
                position: DVec3::ZERO,
            },
        };

        let response = ShipSpawnResponse::new(
            request.id.clone(),
            ShipSpawnResponsePayload {
                success: true,
                identity: bundle.identity.id().to_string(),
                client_authority: bundle.client_authority.id().to_string(),
                server_authority: bundle.server_authority.id().to_string(),
                transformation: bundle.transformation.into(),
            },
        );

        if response.try_send(endpoint).is_err() {
            error!("failed to send response to endpoint");
            return;
        }

        info!(identity =? bundle.identity.id(), "spawned");
        commands.spawn(bundle);
    });
}
