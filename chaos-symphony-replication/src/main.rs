#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

mod authenticate;
mod identities;
mod replicate;
mod ship_spawn;
mod types;

use std::sync::mpsc::TryRecvError;

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    network_disconnect::NetworkDisconnectPlugin,
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};
use chaos_symphony_protocol::{
    AuthenticateRequest, IdentitiesRequest, PingEvent, ReplicateRequest, ShipSpawnRequest,
};
use identities::IdentitiesPlugin;
use replicate::ReplicatePlugin;

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
    .add_plugins((
        NetworkPlugin {
            client: false,
            server: true,
        },
        NetworkDisconnectPlugin,
    ))
    .add_plugins((IdentitiesPlugin, ReplicatePlugin))
    .add_systems(Update, (accepted, route, authenticate::request))
    .add_systems(
        Update,
        (
            ship_spawn::broadcast,
            ship_spawn::callback,
            ship_spawn::replicate,
            ship_spawn::request,
        ),
    );

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

#[allow(clippy::needless_pass_by_value)]
fn route(mut commands: Commands, endpoints: Query<&NetworkEndpoint>) {
    endpoints.for_each(|endpoint| {
        while let Ok(message) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { message } = message;
            match message.endpoint.as_str() {
                PingEvent::ENDPOINT => {
                    // do nothing
                }
                AuthenticateRequest::ENDPOINT => {
                    commands.spawn((
                        NetworkEndpointId {
                            inner: endpoint.id(),
                        },
                        NetworkMessage {
                            inner: AuthenticateRequest::from(message),
                        },
                    ));
                }
                IdentitiesRequest::ENDPOINT => {
                    commands.spawn((
                        NetworkEndpointId {
                            inner: endpoint.id(),
                        },
                        NetworkMessage {
                            inner: IdentitiesRequest::from(message),
                        },
                    ));
                }
                ReplicateRequest::ENDPOINT => {
                    commands.spawn((
                        NetworkEndpointId {
                            inner: endpoint.id(),
                        },
                        NetworkMessage {
                            inner: ReplicateRequest::from(message),
                        },
                    ));
                }
                ShipSpawnRequest::ENDPOINT => {
                    commands.spawn((
                        NetworkEndpointId {
                            inner: endpoint.id(),
                        },
                        NetworkMessage {
                            inner: ShipSpawnRequest::from(message),
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
