use bevy::prelude::*;
use chaos_symphony_protocol::EntityIdentityEvent;

use crate::{network::NetworkMessage, types::EntityIdentity};

/// Entity Identity Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct EntityIdentityPlugin;

impl Plugin for EntityIdentityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, event);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn event(mut commands: Commands, messages: Query<(Entity, &NetworkMessage<EntityIdentityEvent>)>) {
    messages.for_each(|(entity, message)| {
        let message = &message.inner;
        commands.entity(entity).despawn();

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let payload = &message.payload;

        let entity_identity = EntityIdentity {
            inner: payload.inner.clone().into(),
        };
        info!(entity_identity =? entity_identity, "spawned");
        commands.spawn(entity_identity);
    });
}
