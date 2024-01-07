#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

mod entity_identities;
mod entity_identity;
mod network;
mod network_authenticate;
mod replicate_entity_components;

use std::{str::FromStr as _, sync::mpsc::TryRecvError};

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    network_authority::NetworkAuthorityPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    replication,
    types::{EntityIdentity, Identity, NetworkIdentity, Transformation},
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};
use entity_identities::EntityIdentitiesPlugin;
use entity_identity::EntityIdentityPlugin;
use network_authenticate::NetworkAuthenticatePlugin;
use replicate_entity_components::ReplicateEntityComponentsPlugin;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins(BevyConfigPlugin {
        headless: false,
        log_filter: "chaos_symphony_replication".to_string(),
        title: "Chaos Symphony Replication".to_string(),
    })
    // Default Plugins (Network)
    .add_plugins((
        NetworkPlugin {
            client: false,
            server: true,
        },
        NetworkAuthenticatePlugin {
            identity: NetworkIdentity {
                inner: Identity {
                    id: Uuid::from_str("84988f7d-2146-4677-b4f8-6d503f72fea3").unwrap(),
                    noun: "replication".to_string(),
                },
            },
        },
        NetworkAuthorityPlugin,
        NetworkDisconnectPlugin,
    ))
    // Default Plugins
    .add_plugins((
        EntityIdentitiesPlugin,
        EntityIdentityPlugin,
        ReplicateEntityComponentsPlugin,
    ))
    // ...
    .add_systems(Update, (accepted, route));

    // SPIKE IN PROGRESS
    app.add_plugins(replication::ReplicationRequestPlugin);
    app.add_plugins(replication::ReplicationPlugin::<
        Transformation,
        chaos_symphony_protocol::TransformationEvent,
        chaos_symphony_protocol::TransformationEventPayload,
    >::new(replication::ReplicationMode::Replication));

    app.register_type::<EntityIdentity>();
    app.register_type::<Transformation>();

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
