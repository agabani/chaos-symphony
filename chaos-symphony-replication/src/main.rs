#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

mod authenticate;
mod ship_spawn;

use std::sync::mpsc::TryRecvError;

use bevy::{log::LogPlugin, prelude::*};
use chaos_symphony_ecs::network_disconnect::NetworkDisconnectPlugin;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};
use chaos_symphony_protocol::{AuthenticateRequest, ShipSpawnRequest};

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
            level: bevy::log::Level::DEBUG,
        },
    ))
    .add_plugins((
        NetworkPlugin {
            client: false,
            server: true,
        },
        NetworkDisconnectPlugin,
    ))
    .add_systems(
        Update,
        (
            accepted,
            route,
            authenticate::request,
            ship_spawn::callback,
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
        while let Ok(payload) = endpoint.try_recv() {
            let NetworkRecv::NonBlocking { payload } = payload;
            match payload.endpoint.as_str() {
                "/event/ping" => {
                    // do nothing
                }
                "/request/authenticate" => {
                    commands.spawn((
                        EndpointId {
                            inner: endpoint.id(),
                        },
                        Request {
                            inner: AuthenticateRequest::from(payload),
                        },
                    ));
                }
                "/request/ship_spawn" => {
                    commands.spawn((
                        EndpointId {
                            inner: endpoint.id(),
                        },
                        Request {
                            inner: ShipSpawnRequest::from(payload),
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

/// Request.
#[derive(Component)]
struct Request<T> {
    inner: T,
}

/// Endpoint Id.
#[derive(Component)]
struct EndpointId {
    inner: usize,
}
