use bevy::prelude::*;
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::{Connecting, NetworkClient, NetworkEndpoint};

/// Network Connect Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkConnectPlugin;

impl Plugin for NetworkConnectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, initiate));
    }
}

/// Initiate.
///
/// Initiates connections when connection pool drops below 1.
#[allow(clippy::needless_pass_by_value)]
fn initiate(
    mut commands: Commands,
    client: Res<NetworkClient>,
    callbacks: Query<(), With<Connecting>>,
    endpoints: Query<(), With<NetworkEndpoint>>,
) {
    let connections = callbacks.iter().count() + endpoints.iter().count();
    for _ in connections..1 {
        if let Ok(connecting) = client.connect() {
            commands.spawn(connecting);
        } else {
            error!("failed to initiate connect");
        }
    }
}

/// Callback.
///
/// Manages [`Connecting`] lifetime.
///
/// - On success, spawns [`NetworkEndpoint`].
/// - On ready, despawns [`Connecting`].
#[allow(clippy::needless_pass_by_value)]
fn callback(mut commands: Commands, callbacks: Query<(Entity, &Connecting)>) {
    callbacks.for_each(|(entity, callback)| {
        if let Poll::Ready(result) = callback.try_poll() {
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
