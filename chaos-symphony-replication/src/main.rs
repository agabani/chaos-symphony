#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

mod network;

use std::sync::mpsc::TryRecvError;

use bevy::{log::LogPlugin, prelude::*};
use network::{NetworkEndpoint, NetworkPlugin, NetworkServer};

use crate::network::NetworkRecv;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,chaos_symphony_replication=debug".into(),
            level: bevy::log::Level::DEBUG,
        },
    ))
    .add_plugins(NetworkPlugin)
    .add_systems(Update, (accept, disconnected, recv));

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn accept(mut commands: Commands, server: Res<NetworkServer>) {
    loop {
        match server.try_accept() {
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
