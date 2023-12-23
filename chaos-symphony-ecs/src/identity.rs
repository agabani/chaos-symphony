use bevy::prelude::*;
use chaos_symphony_protocol::IdentityEvent;

use crate::{network::NetworkMessage, types::Identity};

/// Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct IdentitiesPlugin;

impl Plugin for IdentitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, event);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn event(
    mut commands: Commands,
    messages: Query<(Entity, &NetworkMessage<IdentityEvent>)>,
    identities: Query<&Identity>,
) {
    messages.for_each(|(entity, message)| {
        commands.entity(entity).despawn();

        let message = &message.inner;

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let identity: Identity = message.payload.identity.clone().into();

        if identities.iter().any(|i| *i == identity) {
            debug!("already spawned");
            return;
        }

        info!(identity =% message.payload.identity, "spawned");
        commands.spawn(identity);
    });
}
