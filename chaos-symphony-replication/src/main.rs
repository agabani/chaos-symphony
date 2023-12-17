#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony Replication

use std::sync::mpsc::TryRecvError;

use bevy::{log::LogPlugin, prelude::*};
use chaos_symphony_ecs::network_disconnect::NetworkDisconnectPlugin;
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv, NetworkServer};
use chaos_symphony_protocol::AuthenticateResponse;

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
    .add_systems(Update, (accepted, recv));

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
fn recv(endpoints: Query<(Entity, &NetworkEndpoint)>) {
    endpoints.for_each(|(entity, endpoint)| {
        let span = info_span!("recv", entity =? entity, id = endpoint.id(), remote_address =% endpoint.remote_address());
        let _guard = span.enter();

        while let Ok(payload) = endpoint.try_recv() {
            match payload {
                NetworkRecv::NonBlocking { payload } => {
                    info!("recv: {payload:?}");

                    if payload.endpoint == "/request/authenticate" {
                        let response = AuthenticateResponse {
                            id: payload.id,
                            success: true
                        };

                        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
                            warn!(error =? error, "unable to send authenticate response");
                        }
                    }
                }
            }
        }
    });
}
