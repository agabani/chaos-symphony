#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony AI

use bevy::{log::LogPlugin, prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    network_authenticate::NetworkAuthenticatePlugin,
    network_connect::NetworkConnectPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    network_keep_alive::NetworkKeepAlivePlugin,
    ship::Ship,
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv};
use chaos_symphony_protocol::{ShipSpawnRequest, ShipSpawning};
use tracing::instrument;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: [
                "info",
                "chaos_symphony_ai=debug",
                "chaos_symphony_ecs=debug",
                "chaos_symphony_network_bevy=debug",
                "wgpu_core=warn",
                "wgpu_hal=warn",
            ]
            .join(","),
            level: bevy::log::Level::DEBUG,
        },
    ))
    .add_plugins((
        NetworkPlugin {
            client: true,
            server: false,
        },
        NetworkAuthenticatePlugin {
            identity: "ai".to_string(),
        },
        NetworkConnectPlugin,
        NetworkDisconnectPlugin,
        NetworkKeepAlivePlugin,
    ))
    .add_systems(Update, (recv, ship_spawn, ship_spawning));

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn recv(endpoints: Query<(Entity, &NetworkEndpoint)>) {
    endpoints.for_each(|(entity, endpoint)| {
        let span = info_span!("recv", entity =? entity, id = endpoint.id(), remote_address =% endpoint.remote_address());
        let _guard = span.enter();

        while let Ok(payload) = endpoint.try_recv() {
            match payload {
                NetworkRecv::NonBlocking { payload } => {
                    info!("recv: {payload:?}");
                }
            }
        }
    });
}

#[instrument(skip_all)]
fn ship_spawn(
    mut commands: Commands,
    endpoints: Query<(&NetworkEndpoint, &ClientAuthority)>,
    ships: Query<(), With<Ship>>,
    ship_spawning: Query<(), With<ShipSpawning>>,
) {
    if let Some((endpoint, client_authority)) = endpoints.iter().next() {
        let count = ships.iter().count() + ship_spawning.iter().count();

        for _ in count..1 {
            let request = ShipSpawnRequest {
                id: Uuid::new_v4().to_string(),
                client_authority: None,
            };

            let span = error_span!("request", request =? request);
            let _guard = span.enter();

            if let Ok(ship_spawning) = request.try_send(endpoint) {
                info!("success");
                commands.spawn((client_authority.clone(), ship_spawning));
            } else {
                error!("failed");
            }
        }
    }
}

#[instrument(skip_all)]
fn ship_spawning(mut commands: Commands, ship_spawnings: Query<(Entity, &ShipSpawning)>) {
    ship_spawnings.for_each(|(entity, ship_spawning)| {
        if let Poll::Ready(result) = ship_spawning.try_poll() {
            let response = match result {
                Ok(response) => {
                    commands.entity(entity).remove::<ShipSpawning>();
                    response
                }
                Err(error) => {
                    error!(error =? error, "network");
                    commands.entity(entity).despawn();
                    return;
                }
            };

            if !response.success {
                error!("failed");
                commands.entity(entity).despawn();
                return;
            }

            let identity = Identity::new(response.identity);
            info!(identity =? identity, "spawned");

            commands.spawn((
                ClientAuthority::new(response.client_authority.unwrap()),
                ServerAuthority::new(response.server_authority.unwrap()),
                identity,
                Ship,
            ));
        }
    });
}
