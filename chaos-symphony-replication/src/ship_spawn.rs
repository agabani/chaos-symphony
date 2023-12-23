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

#[instrument(skip_all)]
pub fn callback(
    mut commands: Commands,
    callbacks: Query<(
        Entity,
        &ShipSpawning,
        &ClientAuthority,
        &ServerAuthority,
        &NetworkEndpointId,
    )>,
    endpoints: Query<&NetworkEndpoint>,
) {
    callbacks.for_each(
        |(entity, callback, client_authority, server_authority, endpoint_id)| {
            let span = error_span!("callback", message_id =% callback.id);
            let _guard = span.enter();

            if let Poll::Ready(result) = callback.try_poll() {
                commands.entity(entity).despawn();

                let endpoint = endpoints
                    .iter()
                    .find(|endpoint| endpoint.id() == endpoint_id.inner);

                let Ok(mut response) = result else {
                    error!("failed to receive response from server");

                    let Some(endpoint) = endpoint else {
                        warn!("client endpoint not found");
                        return;
                    };

                    let response =
                        ShipSpawnResponse::new(callback.id, ShipSpawnResponsePayload::Failure);
                    if response.try_send(endpoint).is_err() {
                        warn!("failed to send response to client endpoint");
                    }

                    return;
                };

                let ShipSpawnResponsePayload::Success(success) = &mut response.payload else {
                    warn!("server rejected request");

                    let Some(endpoint) = endpoint else {
                        warn!("client endpoint not found");
                        return;
                    };

                    if response.try_send(endpoint).is_err() {
                        warn!("failed to send response to client endpoint");
                    }

                    return;
                };

                info!(identity =? success.identity, "spawned");
                let identity: Identity = success.identity.clone().into();

                commands.spawn((identity, client_authority.clone(), server_authority.clone()));

                let Some(endpoint) = endpoint else {
                    warn!("client endpoint not found");
                    return;
                };

                if response.try_send(endpoint).is_err() {
                    warn!("failed to send response to client endpoint");
                }
            }
        },
    );
}

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
            NetworkEndpointId {
                inner: client_endpoint.id(),
            },
            callback,
            client_authority.clone(),
            server_authority.clone(),
        ));
    });
}
