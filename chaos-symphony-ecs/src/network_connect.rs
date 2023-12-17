use bevy::prelude::*;
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::{Connecting, NetworkClient, NetworkEndpoint};

/// Network Connect Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkConnectPlugin;

impl Plugin for NetworkConnectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (connect, connecting));
    }
}

/// Connect.
///
/// Initiates connections when connection pool drops below 1.
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

/// Connecting.
///
/// Manages [`Connecting`] lifetime
///
/// - On success, spawns [`NetworkEndpoint`].
/// - On ready, despawns [`Connecting`].
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
