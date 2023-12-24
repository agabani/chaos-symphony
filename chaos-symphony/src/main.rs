#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony

mod transformation;

use std::str::FromStr as _;

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_ecs::{network, types::Identity};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkRecv};

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
    .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
    .add_plugins(chaos_symphony_ecs::DefaultPlugins {
        identity: Identity::new(
            "client".to_string(),
            Uuid::from_str("0d9aa2b8-0860-42c2-aa20-c2e66dac32b4").unwrap(),
        ),
    })
    .add_plugins(transformation::TransformationPlugin)
    .add_systems(Startup, camera)
    .add_systems(Update, route);

    app.run();
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[allow(clippy::match_single_binding)]
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
