use bevy::prelude::*;
use chaos_symphony_protocol::ShipEvent;

use crate::{
    network::NetworkMessage,
    transform::Transformation,
    types::{ClientAuthority, Identity, ServerAuthority},
};

/// Ship.
#[derive(Component)]
pub struct Ship;

/// Ship Bundle.
#[allow(clippy::module_name_repetitions)]
#[derive(Bundle)]
pub struct ShipBundle {
    /// Ship.
    pub ship: Ship,

    /// Identity.
    pub identity: Identity,

    /// Client Authority.
    pub client_authority: ClientAuthority,

    /// Server Authority.
    pub server_authority: ServerAuthority,

    /// Transform.
    pub transformation: Transformation,
}

/// Ship Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, event);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn event(
    mut commands: Commands,
    messages: Query<(Entity, &NetworkMessage<ShipEvent>)>,
    identities: Query<(Entity, &Identity)>,
) {
    messages.for_each(|(entity, message)| {
        commands.entity(entity).despawn();

        let message = &message.inner;

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let identity: Identity = message.payload.identity.clone().into();

        let Some((entity, _)) = identities.iter().find(|(_, i)| **i == identity) else {
            warn!("identity not found");
            return;
        };

        info!(identity =% message.payload.identity, "inserted");
        commands.entity(entity).insert(Ship);
    });
}
