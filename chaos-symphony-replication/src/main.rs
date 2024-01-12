#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

use std::{str::FromStr as _, sync::mpsc::TryRecvError};

use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    bevy_config::BevyConfigPlugin,
    entity_identities::EntityIdentitiesPlugin,
    entity_identity::EntityIdentityPlugin,
    network_authenticate::NetworkAuthenticatePlugin,
    network_authority::NetworkAuthorityPlugin,
    network_disconnect::NetworkDisconnectPlugin,
    network_router::NetworkRouter,
    replicate_entity_components::ReplicateEntityComponentsPlugin,
    replication::{ReplicationPlugin, ReplicationRequestPlugin},
    types::{EntityIdentity, Identity, NetworkIdentity, Role, Transformation},
};
use chaos_symphony_network_bevy::{NetworkPlugin, NetworkServer};
use chaos_symphony_protocol::{TransformationEvent, TransformationEventPayload};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    let role = Role::Replication;

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
            role,
        },
        NetworkAuthorityPlugin,
        NetworkDisconnectPlugin,
        NetworkRouter,
    ))
    // Default Plugins
    .add_plugins((
        EntityIdentitiesPlugin::new(role),
        EntityIdentityPlugin::new(role),
        ReplicateEntityComponentsPlugin::new(role),
    ))
    .add_systems(Update, accepted);
    // ...

    // SPIKE IN PROGRESS
    app.add_plugins(ReplicationRequestPlugin);
    app.add_plugins(ReplicationPlugin::<
        Transformation,
        TransformationEvent,
        TransformationEventPayload,
    >::new(role));

    app.register_type::<EntityIdentity>();
    app.register_type::<NetworkIdentity>();
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
