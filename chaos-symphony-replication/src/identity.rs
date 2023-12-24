use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::types::{EntityClientAuthority, EntityServerAuthority, Identity};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{IdentityEvent, IdentityEventPayload};

/// Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct IdentityPlugin;

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, broadcast);
    }
}

fn broadcast(
    identities: Query<&Identity, Added<Identity>>,
    server_endpoints: Query<&NetworkEndpoint, With<EntityServerAuthority>>,
    client_endpoints: Query<&NetworkEndpoint, With<EntityClientAuthority>>,
) {
    identities.for_each(|identity| {
        let endpoints = server_endpoints.iter().chain(client_endpoints.iter());
        endpoints.for_each(|endpoint| {
            let identity = identity.clone().into();

            let span = error_span!("broadcast", identity_id =% identity);
            let _guard = span.enter();

            let message = IdentityEvent::new(Uuid::new_v4(), IdentityEventPayload { identity });
            if message.try_send(endpoint).is_err() {
                warn!("failed to send event");
            }
        });
    });
}
