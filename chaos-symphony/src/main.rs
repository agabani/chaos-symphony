#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony

mod transformation;

use std::str::FromStr as _;

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    network_authenticate::NetworkAuthenticatePlugin,
    types::{Identity, NetworkIdentity, Role},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    let role = Role::Client;

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
            role,
        },
        role,
    })
    .add_plugins(transformation::TransformationPlugin)
    .add_systems(Startup, camera);

    app.run();
}

fn camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
