use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{Event, TransformationEvent, TransformationEventPayload};

use crate::types::{EntityIdentity, NetworkReplicationAuthority, Role, Transformation};

/// Transformation Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct TransformationPlugin {
    role: Role,
}

impl TransformationPlugin {
    /// Creates a new [`TransformationPlugin`].
    #[must_use]
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

impl Plugin for TransformationPlugin {
    #[allow(clippy::single_match)]
    fn build(&self, app: &mut App) {
        match self.role {
            Role::Simulation => {
                app.add_systems(Update, broadcast_on_change);
            }
            _ => {}
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn broadcast_on_change(
    endpoints: Query<&NetworkEndpoint, With<NetworkReplicationAuthority>>,
    query: Query<(&Transformation, &EntityIdentity), Changed<Transformation>>,
) {
    query.for_each(|(transformation, entity_identity)| {
        endpoints.for_each(|endpoint| {
            let message = TransformationEvent::message(
                Uuid::new_v4(),
                TransformationEventPayload {
                    entity_identity: entity_identity.inner.clone().into(),
                    transformation: (*transformation).into(),
                },
            );

            if message.try_send(endpoint).is_err() {
                error!("failed to send event");
            };
        });
    });
}
