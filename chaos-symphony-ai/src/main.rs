#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony AI

use std::str::FromStr as _;

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    network_authenticate::NetworkAuthenticatePlugin,
    replication::ReplicationMode,
    types::{Identity, NetworkIdentity},
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(chaos_symphony_ecs::DefaultPlugins {
        bevy_config: BevyConfigPlugin {
            headless: false,
            log_filter: "chaos_symphony_ai".to_string(),
            title: "Chaos Symphony AI".to_string(),
        },
        network_authenticate: NetworkAuthenticatePlugin {
            identity: NetworkIdentity {
                inner: Identity {
                    id: Uuid::from_str("d908808f-073d-4c57-9c08-bf91ba2b1bce").unwrap(),
                    noun: "ai".to_string(),
                },
            },
        },
        replication_mode: ReplicationMode::Client,
    });

    app.run();
}
