#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony AI

mod ship_spawn;

use std::str::FromStr as _;

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    network_authenticate::NetworkAuthenticatePlugin,
    network_connect::NetworkConnectPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    network_keep_alive::NetworkKeepAlivePlugin,
    ship_spawn::ShipSpawnPlugin,
    types::Identity,
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
            level: Level::DEBUG,
        },
    ))
    .add_plugins((
        NetworkPlugin {
            client: true,
            server: false,
        },
        NetworkAuthenticatePlugin {
            identity: Identity::new(
                "ai".to_string(),
                Uuid::from_str("d908808f-073d-4c57-9c08-bf91ba2b1bce").unwrap(),
            ),
        },
        NetworkConnectPlugin,
        NetworkDisconnectPlugin,
        NetworkKeepAlivePlugin,
    ))
    .add_plugins(ShipSpawnPlugin)
    .add_systems(Update, route)
    .add_systems(Update, (ship_spawn::callback, ship_spawn::request));

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn route(mut commands: Commands, endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;
            match message.endpoint.as_str() {
                ShipSpawnEvent::ENDPOINT => {
                    commands.spawn((
                        NetworkEndpointId {
                            inner: endpoint.id(),
                        },
                        NetworkMessage {
                            inner: ShipSpawnEvent::from(message),
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
