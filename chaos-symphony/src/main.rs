#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony

mod transformation;

use std::str::FromStr as _;

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    network,
    network_authenticate::NetworkAuthenticatePlugin,
    replication::ReplicationMode,
    types::{Identity, NetworkIdentity},
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkRecv};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(chaos_symphony_ecs::DefaultPlugins {
        bevy_config: BevyConfigPlugin {
            headless: false,
            log_filter: "chaos_symphony".to_string(),
            title: "Chaos Symphony".to_string(),
        },
        network_authenticate: NetworkAuthenticatePlugin {
            identity: NetworkIdentity {
                inner: Identity {
                    id: Uuid::from_str("0d9aa2b8-0860-42c2-aa20-c2e66dac32b4").unwrap(),
                    noun: "client".to_string(),
                },
            },
        },
        replication_mode: ReplicationMode::Client,
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
fn route(mut commands: Commands, endpoints: Query<(&NetworkEndpoint, Option<&NetworkIdentity>)>) {
    endpoints.for_each(|(endpoint, identity)| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;
            if let Some(message) = network::route(&mut commands, endpoint, identity, message) {
                match message.endpoint.as_str() {
                    endpoint => {
                        warn!(endpoint, "unhandled");
                    }
                }
            }
        }
    });
}
