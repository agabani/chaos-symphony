#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

mod authenticate;
mod network;

use std::sync::mpsc::TryRecvError;

use authenticate::AuthenticatePlugin;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use chaos_symphony_ecs::network_disconnect::NetworkDisconnectPlugin;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};

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
            level: Level::DEBUG,
        },
    ))
    // Default Plugins (Network)
    .add_plugins((
        NetworkPlugin {
            client: false,
            server: true,
        },
        NetworkDisconnectPlugin,
    ))
    // Default Plugins
    .add_plugins(AuthenticatePlugin)
    // ...
    .add_systems(Update, (accepted, route));

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
fn route(mut commands: Commands, endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;
            if let Some(message) = network::route(&mut commands, endpoint, message) {
                match message.endpoint.as_str() {
                    endpoint => {
                        warn!(endpoint, "unhandled");
                    }
                }
            }
        }
    });
}
