#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony AI

use std::str::FromStr as _;

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    network,
    network_authenticate::NetworkAuthenticatePlugin,
    types::{Identity, NetworkIdentity},
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkRecv};

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
    })
    .add_systems(Update, route);

    app.run();
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
