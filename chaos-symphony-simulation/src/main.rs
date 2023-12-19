#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

use bevy::{log::LogPlugin, prelude::*, utils::Uuid};
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
use chaos_symphony_protocol::{ShipSpawnRequest, ShipSpawnResponse};
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
                "chaos_symphony_simulation=debug",
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
            identity: "simulation".to_string(),
        },
        NetworkConnectPlugin,
        NetworkDisconnectPlugin,
        NetworkKeepAlivePlugin,
    ))
    .add_systems(Update, router);

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
#[instrument(skip_all)]
fn router(mut commands: Commands, endpoints: Query<(&NetworkEndpoint, &ServerAuthority)>) {
    endpoints.for_each(|(endpoint, server_authority)| {
        let span = error_span!(
        "router",
            id = endpoint.id(),
            remote_address =% endpoint.remote_address()
        );
        let _guard = span.enter();

        while let Ok(payload) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { payload } = payload;

            match payload.endpoint.as_str() {
                "/request/ship_spawn" => {
                    let request = ShipSpawnRequest::from(payload);

                    info!(request =? request, "request");

                    let Some(client_authority) = request.client_authority else {
                        error!("no client authority");

                        let response = ShipSpawnResponse {
                            id: request.id,
                            success: false,
                            identity: String::new(),
                            client_authority: None,
                            server_authority: None,
                        };

                        if response.try_send(endpoint).is_err() {
                            warn!("failed to send error");
                        }

                        continue;
                    };

                    let client_authority = ClientAuthority::new(client_authority);
                    let server_authority = server_authority.clone();
                    let identity = Identity::new(Uuid::new_v4().to_string());
                    let ship = Ship;

                    let response = ShipSpawnResponse {
                        id: request.id,
                        success: true,
                        identity: identity.id().to_string(),
                        client_authority: None,
                        server_authority: None,
                    };

                    if response.try_send(endpoint).is_err() {
                        warn!("failed to send success");
                        continue;
                    }

                    info!(identity =? identity, "spawned");
                    commands.spawn((client_authority, server_authority, identity, ship));
                }
                endpoint => {
                    warn!(endpoint, "unhandled");
                }
            }
        }
    });
}
