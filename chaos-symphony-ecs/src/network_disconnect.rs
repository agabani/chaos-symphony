use bevy::prelude::*;
use chaos_symphony_network_bevy::NetworkEndpoint;

/// Network Disconnect Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkDisconnectPlugin;

impl Plugin for NetworkDisconnectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, disconnected);
    }
}

/// Disconnected.
///
/// Despawns disconnected [`NetworkEndpoint`].
#[allow(clippy::needless_pass_by_value)]
fn disconnected(mut commands: Commands, endpoints: Query<(Entity, &NetworkEndpoint)>) {
    endpoints.for_each(|(entity, endpoint)| {
        let span = info_span!("disconnected", entity =? entity, id = endpoint.id(), remote_address =% endpoint.remote_address());
        let _guard = span.enter();

        if endpoint.is_disconnected() {
            commands.entity(entity).despawn();
            info!("disconnected");
        }
    });
}
