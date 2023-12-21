#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony AI

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
use chaos_symphony_protocol::ShipSpawnEvent;

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
    .add_systems(Update, route)
    .add_systems(
        Update,
        (ship_spawn::callback, ship_spawn::event, ship_spawn::request),
    );

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn route(mut commands: Commands, endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(payload) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { payload } = payload;
            match payload.endpoint.as_str() {
                "/event/ship_spawn" => {
                    commands.spawn((
                        EndpointId {
                            inner: endpoint.id(),
                        },
                        Request {
                            inner: ShipSpawnEvent::from(payload),
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
