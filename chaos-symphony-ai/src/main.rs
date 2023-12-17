#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony AI

use bevy::{log::LogPlugin, prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    network_connect::NetworkConnectPlugin, network_keep_alive::NetworkKeepAlivePlugin,
};
use chaos_symphony_network_bevy::{NetworkEndpoint, NetworkPlugin, NetworkRecv};
use chaos_symphony_protocol::{AuthenticateRequest, Authenticating};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: [
                "info",
                "chaos_symphony_ai=debug",
                "chaos_symphony_ecs=debug",
                "chaos_symphony_bevy_network=debug",
                "wgpu_core=warn",
                "wgpu_hal=warn",
            ]
            .join(","),
            level: bevy::log::Level::DEBUG,
        },
    ))
    .add_plugins((
        NetworkPlugin {
            client: true,
            server: false,
        },
        NetworkConnectPlugin,
        NetworkKeepAlivePlugin,
    ))
    .add_systems(Update, (authenticate, authenticating, recv));

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn authenticate(
    mut commands: Commands,
    endpoints: Query<&NetworkEndpoint, Added<NetworkEndpoint>>,
) {
    endpoints.for_each(|endpoint| {
        let request = AuthenticateRequest {
            id: Uuid::new_v4().to_string(),
            identity: "ai".to_string(),
        };

        match request.try_send(endpoint) {
            Ok(authenticating) => {
                commands.spawn(authenticating);
            }
            Err(error) => {
                warn!(error =? error, "unable to send authenticate request");
            }
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn authenticating(mut commands: Commands, authenticatings: Query<(Entity, &Authenticating)>) {
    authenticatings.for_each(|(entity, authenticating)| {
        if let Poll::Ready(result) = authenticating.try_poll() {
            commands.entity(entity).despawn();

            let response = match result {
                Ok(result) => result,
                Err(error) => {
                    error!(error =? error, "failed to authenticate");
                    return;
                }
            };

            info!(
                id = response.id,
                success = response.success,
                "authenticating"
            );
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
                NetworkRecv::NonBlocking { payload } => {
                    info!("recv: {payload:?}");
                }
            }
        }
    });
}
