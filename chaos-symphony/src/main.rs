#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony

mod ship;
mod transformation;

use std::str::FromStr as _;

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    utils::Uuid,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    identities::IdentitiesPlugin,
    identity::Identity,
    network::{self, NetworkEndpointId, NetworkMessage},
    network_authenticate::NetworkAuthenticatePlugin,
    network_connect::NetworkConnectPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    network_keep_alive::NetworkKeepAlivePlugin,
    ship_spawn::ShipSpawnPlugin,
    transform::Transformation,
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv};
use chaos_symphony_protocol::ShipSpawnEvent;
use ship::ShipPlugin;

use crate::transformation::TransformationPlugin;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins.set(LogPlugin {
            filter: [
                "info",
                "chaos_symphony_ecs=debug",
                "chaos_symphony_network_bevy=debug",
                "chaos_symphony=debug",
                "wgpu_core=warn",
                "wgpu_hal=warn",
            ]
            .join(","),
            level: Level::DEBUG,
        }),
    )
    .add_plugins(WorldInspectorPlugin::new())
    .add_plugins((
        NetworkPlugin {
            client: true,
            server: false,
        },
        NetworkAuthenticatePlugin {
            identity: Identity::new(
                "client".to_string(),
                Uuid::from_str("0d9aa2b8-0860-42c2-aa20-c2e66dac32b4").unwrap(),
            ),
        },
        NetworkConnectPlugin,
        NetworkDisconnectPlugin,
        NetworkKeepAlivePlugin,
    ))
    .add_plugins(IdentitiesPlugin)
    .add_plugins(ShipPlugin)
    .add_plugins(ShipSpawnPlugin)
    .add_plugins(TransformationPlugin)
    .add_systems(Startup, camera)
    .add_systems(Update, route);

    app.register_type::<ClientAuthority>()
        .register_type::<ServerAuthority>()
        .register_type::<Identity>()
        .register_type::<Transformation>()
        .register_type::<Uuid>();

    app.run();
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[allow(clippy::needless_pass_by_value)]
fn route(mut commands: Commands, endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;

            let Some(message) = network::route(&mut commands, endpoint, message) else {
                continue;
            };

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
