use bevy::prelude::*;
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    types::{ClientAuthority, Identity, ServerAuthority},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ShipSpawnRequest, ShipSpawnResponse, ShipSpawnResponsePayload, ShipSpawning,
};
use tracing::instrument;

use crate::types::NetworkPath;

#[instrument(skip_all)]
pub fn request(
    mut commands: Commands,
    messages: Query<(
        Entity,
        &NetworkEndpointId,
        &NetworkMessage<ShipSpawnRequest>,
    )>,
    clients: Query<(&NetworkEndpoint, Option<&ClientAuthority>)>,
    servers: Query<(&NetworkEndpoint, &ServerAuthority)>,
) {
    messages.for_each(|(entity, endpoint_id, message)| {
        let span = error_span!("request", message_id =% message.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some((client_endpoint, client_authority)) = clients
            .iter()
            .find(|(endpoint, _)| endpoint.id() == endpoint_id.inner)
        else {
            warn!("client endpoint not found");
            return;
        };

        let request = &message.inner;

        let Some(client_authority) = client_authority else {
            warn!("client endpoint unauthenticated");
            let response = ShipSpawnResponse::new(request.id, ShipSpawnResponsePayload::Failure);
            if response.try_send(client_endpoint).is_err() {
                warn!("failed to send response to client");
            }
            return;
        };

        let Some((server_endpoint, server_authority)) = servers.iter().next() else {
            warn!("server unavailable");
            let response = ShipSpawnResponse::new(request.id, ShipSpawnResponsePayload::Failure);
            if response.try_send(client_endpoint).is_err() {
                warn!("failed to send response to client");
            }
            return;
        };

        // overwrite
        let id = request.id;

        let mut request = request.clone();
        request.payload.client_authority = Some(client_authority.identity().clone().into());
        request.payload.server_authority = Some(server_authority.identity().clone().into());

        let Ok(callback) = request.try_send(server_endpoint) else {
            error!("failed to send request to server");
            let response = ShipSpawnResponse::new(id, ShipSpawnResponsePayload::Failure);
            if response.try_send(client_endpoint).is_err() {
                warn!("failed to send response to client");
            }
            return;
        };

        info!("sent request to server");
        commands.spawn((
            NetworkPath {
                source: NetworkEndpointId {
                    inner: client_endpoint.id(),
                },
                target: NetworkEndpointId {
                    inner: server_endpoint.id(),
                },
            },
            callback,
            client_authority.clone(),
            server_authority.clone(),
        ));
    });
}
