use bevy::prelude::*;
use chaos_symphony_protocol::TransformationEvent;

use crate::{network::NetworkMessage, transform::Transformation, types::Identity};

/// Transformation Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct TransformationPlugin;

impl Plugin for TransformationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, event);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn event(
    mut commands: Commands,
    messages: Query<(Entity, &NetworkMessage<TransformationEvent>)>,
    identities: Query<(Entity, &Identity)>,
) {
    messages.for_each(|(entity, message)| {
        commands.entity(entity).despawn();

        let message = &message.inner;

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let identity: Identity = message.payload.identity.clone().into();
        let transformation: Transformation = message.payload.transformation.clone().into();

        let Some((entity, _)) = identities.iter().find(|(_, i)| **i == identity) else {
            warn!("identity not found");
            return;
        };

        info!(identity =% message.payload.identity, "inserted");
        commands.entity(entity).insert(transformation);
    });
}
