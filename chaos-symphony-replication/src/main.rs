#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

use std::sync::mpsc::TryRecvError;

use bevy::{log::LogPlugin, prelude::*};
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    network_disconnect::NetworkDisconnectPlugin,
    ship::Ship,
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};
use chaos_symphony_protocol::{
    AuthenticateRequest, AuthenticateResponse, ShipSpawnRequest, ShipSpawnResponse, ShipSpawning,
};
use tracing::instrument;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: [
                "info",
                "chaos_symphony_ecs=debug",
                "chaos_symphony_network_bevy=debug",
                "chaos_symphony_replication=debug",
                "wgpu_core=warn",
                "wgpu_hal=warn",
            ]
            .join(","),
            level: bevy::log::Level::DEBUG,
        },
    ))
    .add_plugins((
        NetworkPlugin {
            client: false,
            server: true,
        },
        NetworkDisconnectPlugin,
    ))
    .add_systems(Update, (accepted, router, ship_spawning));

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn accepted(mut commands: Commands, server: Res<NetworkServer>) {
    loop {
        match server.try_recv() {
            Ok(endpoint) => {
                let id = endpoint.id();
                let remote_address = endpoint.remote_address();

                let entity = commands.spawn(endpoint).id();

                let span =
                    info_span!("accept", entity =? entity, id, remote_address =% remote_address);
                let _guard = span.enter();
                info!("connected");
            }
            Err(TryRecvError::Disconnected) => {
                panic!("[network:server] disconnected");
            }
            Err(TryRecvError::Empty) => {
                return;
            }
        };
    }
}

#[allow(clippy::needless_pass_by_value)]
#[instrument(skip_all)]
fn router(
    mut commands: Commands,
    endpoints: Query<(Entity, &NetworkEndpoint, Option<&ClientAuthority>)>,
    servers: Query<(&NetworkEndpoint, &ServerAuthority)>,
) {
    endpoints.for_each(|(entity, endpoint, client_authority)| {
        let span = error_span!(
        "router",
            id = endpoint.id(),
            remote_address =% endpoint.remote_address()
        );
        let _guard = span.enter();

        while let Ok(payload) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { payload } = payload;

            match payload.endpoint.as_str() {
                "/event/ping" => {
                    // do nothing
                }
                "/request/authenticate" => {
                    let request = AuthenticateRequest::from(payload);

                    let identity = request.identity;

                    let response = AuthenticateResponse {
                        id: request.id,
                        success: true,
                        identity: identity.clone(),
                    };

                    if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
                        warn!(error =? error, "unable to send authenticate response");
                    }

                    match identity.as_str() {
                        "ai" | "client" => {
                            commands
                                .entity(entity)
                                .insert(ClientAuthority::new(identity));
                        }
                        "simulation" => {
                            commands
                                .entity(entity)
                                .insert(ServerAuthority::new(identity));
                        }
                        identity => todo!("{identity}"),
                    };
                }
                "/request/ship_spawn" => {
                    let request = ShipSpawnRequest::from(payload);

                    info!(request =? request, "request");

                    let Some(client_authority) = client_authority else {
                        warn!("unauthenticated");

                        let response = ShipSpawnResponse {
                            id: request.id,
                            success: false,
                            identity: String::new(),
                            client_authority: None,
                            server_authority: None,
                        };

                        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
                            warn!(error =? error, "failed to send response");
                        }

                        continue;
                    };

                    let request = request.with_client_authority(client_authority.id().to_string());

                    let Some((server, server_authority)) = servers.iter().next() else {
                        warn!("no servers available");

                        let response = ShipSpawnResponse {
                            id: request.id,
                            success: false,
                            identity: String::new(),
                            client_authority: None,
                            server_authority: None,
                        };

                        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
                            warn!(error =? error, "failed to send response");
                        }

                        continue;
                    };

                    let id = request.id.clone();
                    let Ok(requesting) = request.try_send(&server) else {
                        warn!("failed to delegate request");

                        let response = ShipSpawnResponse {
                            id,
                            success: false,
                            identity: String::new(),
                            client_authority: None,
                            server_authority: None,
                        };

                        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
                            warn!(error =? error, "failed to send response");
                        }

                        continue;
                    };

                    info!("delegated request");
                    commands.spawn((
                        client_authority.clone(),
                        server_authority.clone(),
                        requesting,
                    ));
                }
                endpoint => {
                    warn!(endpoint, "unhandled");
                }
            };
        }
    });
}

#[instrument(skip_all)]
fn ship_spawning(
    mut commands: Commands,
    ship_spawnings: Query<(Entity, &ShipSpawning, &ClientAuthority, &ServerAuthority)>,
    clients: Query<(&NetworkEndpoint, &ClientAuthority)>,
) {
    ship_spawnings.for_each(
        |(entity, ship_spawning, client_authority, server_authority)| {
            if let Poll::Ready(result) = ship_spawning.try_poll() {
                commands.entity(entity).despawn();

                let response = match result {
                    Ok(response) => response,
                    Err(error) => {
                        error!(error =? error, "network");
                        return;
                    }
                };

                if !response.success {
                    error!("failed");
                    return;
                }

                let response = response
                    .with_client_authority(client_authority.id().to_string())
                    .with_server_authority(server_authority.id().to_string());

                let identity = Identity::new(response.identity.clone());

                info!(identity =? identity, "spawned");
                commands.spawn((
                    client_authority.clone(),
                    server_authority.clone(),
                    Identity::new(response.identity.clone()),
                    Ship,
                ));

                let Some((client, _)) = clients
                    .iter()
                    .find(|(_, c)| c.id() == client_authority.id())
                else {
                    warn!("client disconnected");
                    return;
                };

                if response.try_send(client).is_err() {
                    warn!("failed to send success");
                }
            }
        },
    );
}
