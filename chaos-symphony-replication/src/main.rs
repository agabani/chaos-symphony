#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

mod entity_identities;
mod entity_identity;
mod network;
mod network_authenticate;
mod replicate_entity_components;

use std::sync::mpsc::TryRecvError;

use bevy::{
    log::{Level, LogPlugin},
    math::{DQuat, DVec3},
    prelude::*,
    utils::Uuid,
};
use chaos_symphony_ecs::{
    network_authority::NetworkAuthorityPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    transformation::TransformationPlugin,
    types::{EntityIdentity, Identity, Transformation},
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};
use entity_identities::EntityIdentitiesPlugin;
use entity_identity::EntityIdentityPlugin;
use network_authenticate::NetworkAuthenticatePlugin;
use replicate_entity_components::ReplicateEntityComponentsPlugin;

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
        NetworkAuthenticatePlugin,
        NetworkAuthorityPlugin,
        NetworkDisconnectPlugin,
    ))
    // Default Plugins
    .add_plugins((
        EntityIdentitiesPlugin,
        EntityIdentityPlugin,
        ReplicateEntityComponentsPlugin,
        TransformationPlugin,
    ))
    // ...
    .add_systems(Update, (accepted, route))
    .add_systems(Startup, testing);

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

fn testing(mut commands: Commands) {
    commands.spawn((
        EntityIdentity {
            inner: Identity {
                id: Uuid::new_v4(),
                noun: "test_replication".to_string(),
            },
        },
        Transformation {
            orientation: DQuat::from_rotation_z(0.0),
            position: DVec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
    ));
}
