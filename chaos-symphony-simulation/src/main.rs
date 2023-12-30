#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

use std::str::FromStr as _;

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    network,
    network_authenticate::NetworkAuthenticatePlugin,
    replication::ReplicationMode,
    types::{EntityIdentity, Identity, NetworkIdentity, ReplicateSource},
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkRecv};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(chaos_symphony_ecs::DefaultPlugins {
        bevy_config: BevyConfigPlugin {
            headless: false,
            log_filter: "chaos_symphony_simulation".to_string(),
            title: "Chaos Symphony Simulation".to_string(),
        },
        network_authenticate: NetworkAuthenticatePlugin {
            identity: NetworkIdentity {
                inner: Identity {
                    id: Uuid::from_str("d86cb791-fe2f-4f50-85b9-57532d14f037").unwrap(),
                    noun: "simulation".to_string(),
                },
            },
        },
        replication_mode: ReplicationMode::Simulation,
    })
    .add_systems(
        Update,
        (route, test_spawn_entity_identity_after_network_authenticate),
    );

    app.run();
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

#[allow(clippy::needless_pass_by_value)]
fn test_spawn_entity_identity_after_network_authenticate(
    mut commands: Commands,
    query: Query<(), (With<NetworkEndpoint>, Added<NetworkIdentity>)>,
) {
    query.for_each(|()| {
        commands.spawn((
            EntityIdentity {
                inner: Identity {
                    id: Uuid::new_v4(),
                    noun: "test_simulation".to_string(),
                },
            },
            ReplicateSource,
        ));
    });
}
