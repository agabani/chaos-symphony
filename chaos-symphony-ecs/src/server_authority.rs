use bevy::prelude::*;
use chaos_symphony_protocol::ServerAuthorityEvent;

use crate::{
    network::NetworkMessage,
    types::{EntityServerAuthority, Identity},
};

/// Server Authority Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ServerAuthorityPlugin;

impl Plugin for ServerAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, event);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn event(
    mut commands: Commands,
    messages: Query<(Entity, &NetworkMessage<ServerAuthorityEvent>)>,
    identities: Query<(Entity, &Identity)>,
) {
    messages.for_each(|(entity, message)| {
        commands.entity(entity).despawn();

        let message = &message.inner;

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let identity: Identity = message.payload.identity.clone().into();
        let authority: Identity = message.payload.authority.clone().into();

        let Some((entity, _)) = identities.iter().find(|(_, i)| **i == identity) else {
            warn!("identity not found");
            return;
        };

        info!(identity =% message.payload.identity, "inserted");
        commands
            .entity(entity)
            .insert(EntityServerAuthority::new(authority));
    });
}
