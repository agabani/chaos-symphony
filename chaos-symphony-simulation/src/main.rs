#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Simulation

use bevy::{log::LogPlugin, prelude::*};
use chaos_symphony_bevy_network::{
    Connecting, NetworkClient, NetworkEndpoint, NetworkPlugin, NetworkRecv, Poll,
};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: [
                "info",
                "chaos_symphony_bevy_network=debug",
                "chaos_symphony_simulation=debug",
                "wgpu_core=warn",
                "wgpu_hal=warn",
            ]
            .join(","),
            level: bevy::log::Level::DEBUG,
        },
    ))
    .add_plugins(NetworkPlugin {
        client: true,
        server: false,
    })
    .add_systems(Startup, connect)
    .add_systems(Update, (connecting, disconnected, recv));

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn connect(mut commands: Commands, client: Res<NetworkClient>) {
    if let Ok(connecting) = client.connect() {
        commands.spawn(connecting);
    } else {
        error!("failed to initiate connect");
    }
}

#[allow(clippy::needless_pass_by_value)]
fn connecting(mut commands: Commands, connectings: Query<(Entity, &Connecting)>) {
    connectings.for_each(|(entity, connecting)| {
        if let Poll::Ready(result) = connecting.try_poll() {
            commands.entity(entity).despawn();
            match result {
                Ok(endpoint) => {
                    let id = endpoint.id();
                    let remote_address = endpoint.remote_address();

                    let entity = commands.spawn(endpoint).id();

                    let span = info_span!("connecting", entity =? entity, id, remote_address =% remote_address);
                    let _guard = span.enter();
                    info!("connected");
                }
                Err(error) => {
                    error!(error =? error, "failed to connect");
                }
            }
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn disconnected(mut commands: Commands, endpoints: Query<(Entity, &NetworkEndpoint)>) {
    endpoints.for_each(|(entity, endpoint)| {
        let span = info_span!("disconnected", entity =? entity, id = endpoint.id(), remote_address =% endpoint.remote_address());
        let _guard = span.enter();

        if endpoint.is_disconnected() {
            commands.entity(entity).despawn_recursive();
            info!("disconnected");
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn recv(endpoints: Query<(Entity, &NetworkEndpoint)>) {
    endpoints.for_each(|(entity, endpoint)| {
        let span = info_span!("recv", entity =? entity, id = endpoint.id(), remote_address =% endpoint.remote_address());
        let _guard = span.enter();

        while let Ok(payload) = endpoint.try_recv() {
            match payload {
                NetworkRecv::Event(payload) => {
                    info!("recv: {payload:?}");
                }
            }
        }
    });
}
