use bevy::prelude::*;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::ShipSpawnRequest;

use crate::types::{
    EntityIdentity, EntitySimulationAuthority, NetworkIdentity, NetworkSimulationAuthority, Role,
    Untrusted,
};

/// Delegation Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct DelegationPlugin {
    role: Role,
}

impl DelegationPlugin {
    /// Creates a new [`DelegationPlugin`].
    #[must_use]
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

impl Plugin for DelegationPlugin {
    #[allow(clippy::single_match)]
    fn build(&self, app: &mut App) {
        app.add_event::<Untrusted<ShipSpawnRequest>>();

        match self.role {
            Role::Replication => {
                app.add_systems(Update, request);
            }
            _ => {}
        }
    }
}

fn request(
    mut reader: EventReader<Untrusted<ShipSpawnRequest>>,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity), With<NetworkSimulationAuthority>>,
    entities: Query<(&EntityIdentity, Option<&EntitySimulationAuthority>)>,
) {
    reader.read().for_each(|request| {
        let (entity_identity, entity_authority) =
            match entities.iter().find(|(entity_identity, _)| {
                entity_identity.inner == request.inner.payload.entity_identity
            }) {
                Some((identity, simulation_authority)) => (Some(identity), simulation_authority),
                None => (None, None),
            };

        if entity_identity.is_none() {
            // find any endpoint endpoint
        }
    });
}
