#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony

use bevy::{log::LogPlugin, prelude::*};
use chaos_symphony_ecs::{
    network_authenticate::NetworkAuthenticatePlugin, network_connect::NetworkConnectPlugin,
    network_disconnect::NetworkDisconnectPlugin, network_keep_alive::NetworkKeepAlivePlugin,
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((DefaultPlugins.set(LogPlugin {
        filter: [
            "info",
            "chaos_symphony_ecs=debug",
            "chaos_symphony_network_bevy=debug",
            "chaos_symphony=debug",
            "wgpu_core=warn",
            "wgpu_hal=warn",
        ]
        .join(","),
        level: bevy::log::Level::DEBUG,
    }),))
        .add_plugins((
            NetworkPlugin {
                client: true,
                server: false,
            },
            NetworkAuthenticatePlugin {
                identity: "client".to_string(),
            },
            NetworkConnectPlugin,
            NetworkDisconnectPlugin,
            NetworkKeepAlivePlugin,
        ))
        .add_systems(Update, route);

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn route(endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(payload) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { payload } = payload;
            warn!(endpoint = payload.endpoint, "unhandled");
        }
    });
}
