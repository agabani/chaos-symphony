#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony AI

use bevy::{log::LogPlugin, prelude::*, utils::Uuid};
use chaos_symphony_async::{Future, Poll, PollError};
use chaos_symphony_bevy_network::{
    Connecting, NetworkClient, NetworkEndpoint, NetworkPlugin, NetworkRecv,
};
use chaos_symphony_network::Payload;
use chaos_symphony_protocol::{AuthenticateRequest, AuthenticateResponse};

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        LogPlugin {
            filter: [
                "info",
                "chaos_symphony_bevy_network=debug",
                "chaos_symphony_ai=debug",
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
    .add_systems(
        Update,
        (
            authenticate,
            authenticating,
            connect,
            connecting,
            disconnected,
            recv,
        ),
    );

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

        match endpoint.try_send_blocking(request.into()) {
            Ok(future) => {
                commands.spawn(Authenticating { inner: future });
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
fn connect(
    mut commands: Commands,
    client: Res<NetworkClient>,
    connectings: Query<(), With<Connecting>>,
    endpoints: Query<(), With<NetworkEndpoint>>,
) {
    let connections = connectings.iter().count() + endpoints.iter().count();
    for _ in connections..1 {
        if let Ok(connecting) = client.connect() {
            commands.spawn(connecting);
        } else {
            error!("failed to initiate connect");
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn connecting(mut commands: Commands, connectings: Query<(Entity, &Connecting)>) {
    connectings.for_each(|(entity, connecting)| {
        if let Poll::Ready(result) = connecting.try_poll() {
            commands.entity(entity).despawn();

            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    error!(error =? error, "failed to connect");
                    return;
                }
            };

            let endpoint = match result {
                Ok(result) => result,
                Err(error) => {
                    error!(error =? error, "failed to connect");
                    return;
                }
            };

            let id = endpoint.id();
            let remote_address = endpoint.remote_address();

            let entity = commands.spawn(endpoint).id();

            let span =
                info_span!("connecting", entity =? entity, id, remote_address =% remote_address);
            let _guard = span.enter();
            info!("connected");
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
                NetworkRecv::NonBlocking { payload } => {
                    info!("recv: {payload:?}");
                }
            }
        }
    });
}

#[derive(Component)]
struct Authenticating {
    inner: Future<Payload>,
}

impl Authenticating {
    fn try_poll(&self) -> Poll<Result<AuthenticateResponse, PollError>> {
        self.inner.try_poll().map(|r| r.map(Into::into))
    }
}
