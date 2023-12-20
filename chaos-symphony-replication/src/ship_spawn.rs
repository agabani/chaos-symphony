use bevy::prelude::*;
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    ship::Ship,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ShipSpawnRequest, ShipSpawnResponse, ShipSpawning};
use tracing::instrument;

use crate::{EndpointId, Request};

#[instrument(skip_all)]
pub fn callback(
    mut commands: Commands,
    ship_spawnings: Query<(
        Entity,
        &ShipSpawning,
        &ClientAuthority,
        &ServerAuthority,
        &EndpointId,
    )>,
    endpoints: Query<&NetworkEndpoint>,
) {
    ship_spawnings.for_each(
        |(entity, ship_spawning, client_authority, server_authority, endpoint_id)| {
            let span = error_span!("callback", request_id = ship_spawning.id);
            let _guard = span.enter();

            if let Poll::Ready(result) = ship_spawning.try_poll() {
                commands.entity(entity).despawn();

                let client_endpoint = endpoints
                    .iter()
                    .find(|endpoint| endpoint.id() == endpoint_id.inner);

                let response = match result {
                    Ok(response) => response,
                    Err(error) => {
                        error!(error =? error, "failed to receive response from server");

                        if let Some(client_endpoint) = client_endpoint {
                            let response = ShipSpawnResponse {
                                id: ship_spawning.id.clone(),
                                success: false,
                                identity: String::new(),
                                client_authority: None,
                                server_authority: None,
                            };

                            if response.try_send(client_endpoint).is_err() {
                                warn!("failed to send response to client");
                            }
                        } else {
                            warn!("client endpoint not found");
                        }

                        return;
                    }
                };

                if !response.success {
                    warn!("server rejected request");

                    if let Some(client_endpoint) = client_endpoint {
                        if response.try_send(client_endpoint).is_err() {
                            warn!("failed to send response to client");
                        } else {
                            warn!("client endpoint not found");
                        }
                    }

                    return;
                }

                let identity = Identity::new(response.identity.clone());

                info!(identity =? identity, "spawned");
                commands.spawn((
                    Ship,
                    identity,
                    client_authority.clone(),
                    server_authority.clone(),
                ));

                let response = response
                    .with_client_authority(client_authority.id().to_string())
                    .with_server_authority(server_authority.id().to_string());

                if let Some(client_endpoint) = client_endpoint {
                    if response.try_send(client_endpoint).is_err() {
                        warn!("failed to send response to client");
                    }
                } else {
                    warn!("client endpoint not found");
                }
            }
        },
    );
}

#[instrument(skip_all)]
pub fn request(
    mut commands: Commands,
    requests: Query<(Entity, &EndpointId, &Request<ShipSpawnRequest>)>,
    clients: Query<(&NetworkEndpoint, Option<&ClientAuthority>)>,
    servers: Query<(&NetworkEndpoint, &ServerAuthority)>,
) {
    requests.for_each(|(entity, endpoint_id, request)| {
        let span = error_span!("request", request_id = request.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some((client_endpoint, client_authority)) = clients
            .iter()
            .find(|(endpoint, _)| endpoint.id() == endpoint_id.inner)
        else {
            warn!("client endpoint not found");
            return;
        };

        let Some(client_authority) = client_authority else {
            warn!("client endpoint unauthenticated");

            let response = ShipSpawnResponse {
                id: request.inner.id.clone(),
                success: false,
                identity: String::new(),
                client_authority: None,
                server_authority: None,
            };

            if let Err(error) = client_endpoint.try_send_non_blocking(response.into()) {
                warn!(error =? error, "failed to send response to client");
            }

            return;
        };

        let Some((server_endpoint, server_authority)) = servers.iter().next() else {
            warn!("server unavailable");

            let response = ShipSpawnResponse {
                id: request.inner.id.clone(),
                success: false,
                identity: String::new(),
                client_authority: None,
                server_authority: None,
            };

            if let Err(error) = client_endpoint.try_send_non_blocking(response.into()) {
                warn!(error =? error, "failed to send response to client");
            }

            return;
        };

        let Ok(ship_spawning) = request
            .inner
            .clone()
            .with_client_authority(client_authority.id().to_string())
            .try_send(server_endpoint)
        else {
            warn!("failed to send request to server");

            let response = ShipSpawnResponse {
                id: request.inner.id.clone(),
                success: false,
                identity: String::new(),
                client_authority: None,
                server_authority: None,
            };

            if let Err(error) = client_endpoint.try_send_non_blocking(response.into()) {
                warn!(error =? error, "failed to send response to client");
            }

            return;
        };

        info!("sent request to server");
        commands.spawn((
            ship_spawning,
            client_authority.clone(),
            server_authority.clone(),
            EndpointId {
                inner: client_endpoint.id(),
            },
        ));
    });
}
