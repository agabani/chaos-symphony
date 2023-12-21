use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    routing::{EndpointId, Request},
    ship::{Ship, ShipBundle},
    transform::Transformation,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ShipSpawnEvent, ShipSpawnRequest, ShipSpawnResponse, ShipSpawning};
use tracing::instrument;

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

                let endpoint = endpoints
                    .iter()
                    .find(|endpoint| endpoint.id() == endpoint_id.inner);

                let Ok(response) = result else {
                    error!("failed to receive response from server");

                    let Some(endpoint) = endpoint else {
                        warn!("client endpoint not found");
                        return;
                    };

                    if ShipSpawnResponse::error(ship_spawning.id.clone())
                        .try_send(endpoint)
                        .is_err()
                    {
                        warn!("failed to send response to client endpoint");
                    }

                    return;
                };

                if !response.success {
                    warn!("server rejected request");

                    let Some(endpoint) = endpoint else {
                        warn!("client endpoint not found");
                        return;
                    };

                    if response.try_send(endpoint).is_err() {
                        warn!("failed to send response to client endpoint");
                    }

                    return;
                }

                let identity = response.identity.clone();
                info!(identity =? identity, "spawned");
                commands.spawn(ShipBundle {
                    ship: Ship,
                    identity: Identity::new(identity),
                    client_authority: client_authority.clone(),
                    server_authority: server_authority.clone(),
                    transformation: Transformation {
                        orientation: DQuat {
                            x: response.orientation_x,
                            y: response.orientation_y,
                            z: response.orientation_z,
                            w: response.orientation_w,
                        },
                        position: DVec3 {
                            x: response.position_x,
                            y: response.position_y,
                            z: response.position_z,
                        },
                    },
                });

                let response = response
                    .with_client_authority(client_authority.id().to_string())
                    .with_server_authority(server_authority.id().to_string());

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

        let request = &request.inner;

        let Some(client_authority) = client_authority else {
            warn!("client endpoint unauthenticated");

            if ShipSpawnResponse::error(request.id.to_string())
                .try_send(client_endpoint)
                .is_err()
            {
                warn!("failed to send response to client");
            }

            return;
        };

        let Some((server_endpoint, server_authority)) = servers.iter().next() else {
            warn!("server unavailable");

            if ShipSpawnResponse::error(request.id.to_string())
                .try_send(client_endpoint)
                .is_err()
            {
                warn!("failed to send response to client");
            }

            return;
        };

        let Ok(ship_spawning) = request
            .clone()
            .with_client_authority(client_authority.id().to_string())
            .with_server_authority(server_authority.id().to_string())
            .try_send(server_endpoint)
        else {
            error!("failed to send request to server");

            if ShipSpawnResponse::error(request.id.to_string())
                .try_send(client_endpoint)
                .is_err()
            {
                warn!("failed to send response to client");
            }

            return;
        };

        info!("sent request to server");
        commands.spawn((
            EndpointId {
                inner: client_endpoint.id(),
            },
            ship_spawning,
            client_authority.clone(),
            server_authority.clone(),
        ));
    });
}

#[allow(clippy::needless_pass_by_value)]
pub fn broadcast(
    ships: Query<
        (
            &Identity,
            &ClientAuthority,
            &ServerAuthority,
            &Transformation,
        ),
        Added<Ship>,
    >,
    client_endpoints: Query<&NetworkEndpoint, With<ClientAuthority>>,
    server_endpoints: Query<&NetworkEndpoint, With<ServerAuthority>>,
) {
    ships.for_each(
        |(identity, client_authority, server_authority, transformation)| {
            let span = error_span!("broadcast", identity_id = identity.id());
            let _guard = span.enter();

            server_endpoints
                .iter()
                .chain(client_endpoints.iter())
                .for_each(|endpoint| {
                    let id = Uuid::new_v4().to_string();

                    let span = error_span!(
                        "broadcast",
                        endpoint_id = endpoint.id(),
                        identity_id = identity.id(),
                        request_id = id
                    );
                    let _guard = span.enter();

                    let event = ShipSpawnEvent {
                        id,
                        identity: identity.id().to_string(),
                        client_authority: client_authority.id().to_string(),
                        server_authority: server_authority.id().to_string(),
                        orientation_x: transformation.orientation.x,
                        orientation_y: transformation.orientation.y,
                        orientation_z: transformation.orientation.z,
                        orientation_w: transformation.orientation.w,
                        position_x: transformation.position.x,
                        position_y: transformation.position.y,
                        position_z: transformation.position.z,
                    };

                    if event.try_send(endpoint).is_err() {
                        warn!("failed to send event to client");
                    }
                });
        },
    );
}

#[allow(clippy::needless_pass_by_value)]
pub fn replicate(
    client_endpoints: Query<&NetworkEndpoint, Added<ClientAuthority>>,
    server_endpoints: Query<&NetworkEndpoint, Added<ServerAuthority>>,
    ships: Query<(
        &Identity,
        &ClientAuthority,
        &ServerAuthority,
        &Transformation,
    )>,
) {
    server_endpoints
        .iter()
        .chain(client_endpoints.iter())
        .for_each(|endpoint| {
            let span = error_span!("replicate", endpoint_id = endpoint.id());
            let _guard = span.enter();

            ships.for_each(
                |(identity, client_authority, server_authority, transformation)| {
                    let id = Uuid::new_v4().to_string();

                    let span = error_span!(
                        "replicate",
                        endpoint_id = endpoint.id(),
                        identity_id = identity.id(),
                        request_id = id
                    );
                    let _guard = span.enter();

                    let event = ShipSpawnEvent {
                        id,
                        identity: identity.id().to_string(),
                        client_authority: client_authority.id().to_string(),
                        server_authority: server_authority.id().to_string(),
                        orientation_x: transformation.orientation.x,
                        orientation_y: transformation.orientation.y,
                        orientation_z: transformation.orientation.z,
                        orientation_w: transformation.orientation.w,
                        position_x: transformation.position.x,
                        position_y: transformation.position.y,
                        position_z: transformation.position.z,
                    };

                    if event.try_send(endpoint).is_err() {
                        warn!("failed to send event to client");
                    }
                },
            );
        });
}
