#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

mod ship_spawn;

use bevy::{log::LogPlugin, prelude::*};
use chaos_symphony_ecs::{
    network_authenticate::NetworkAuthenticatePlugin,
    network_connect::NetworkConnectPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    network_keep_alive::NetworkKeepAlivePlugin,
    routing::{EndpointId, Request},
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv};
use chaos_symphony_protocol::ShipSpawnRequest;

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
    .add_systems(Update, (route, ship_spawn::request));

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn route(mut commands: Commands, endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(payload) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { payload } = payload;
            match payload.endpoint.as_str() {
                "/request/ship_spawn" => {
                    commands.spawn((
                        EndpointId {
                            inner: endpoint.id(),
                        },
                        Request {
                            inner: ShipSpawnRequest::from(payload),
                        },
                    ));
                }
                endpoint => {
                    warn!(endpoint, "unhandled");
                }
            }
        }
    });
}
