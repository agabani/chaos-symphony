use bevy::prelude::*;
use chaos_symphony_network_bevy::NetworkEndpoint;

use crate::types::{NetworkClientAuthority, NetworkIdentity, NetworkServerAuthority};

/// Network Authority Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkAuthorityPlugin;

impl Plugin for NetworkAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, label);
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
fn label(
    mut commands: Commands,
    endpoints: Query<(Entity, &NetworkIdentity), (With<NetworkEndpoint>, Added<NetworkIdentity>)>,
) {
    endpoints.for_each(|(entity, identity)| match identity.inner.noun.as_str() {
        "ai" | "client" => {
            commands.entity(entity).insert(NetworkClientAuthority);
        }
        "simulation" => {
            commands.entity(entity).insert(NetworkServerAuthority);
        }
        noun => {
            error!(identity_noun = noun, "unrecognized");
        }
    });
}
