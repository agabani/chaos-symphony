use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    routing::{EndpointId, Request},
    ship::{Ship, ShipBundle},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ShipSpawnRequest, ShipSpawnResponse};
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

        let identity = Uuid::new_v4().to_string();

        let response = ShipSpawnResponse {
            id: request.id.clone(),
            success: true,
            identity: identity.clone(),
            client_authority: request.client_authority.clone(),
            server_authority: request.server_authority.clone(),
        };

        if response.try_send(endpoint).is_err() {
            error!("failed to send response to endpoint");
            return;
        }

        info!(identity =? identity, "spawned");
        commands.spawn(ShipBundle {
            ship: Ship,
            identity: Identity::new(identity),
            client_authority: ClientAuthority::new(request.client_authority.clone()),
            server_authority: ServerAuthority::new(request.server_authority.clone()),
        });
    });
}
