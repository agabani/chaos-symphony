use bevy::prelude::*;
use chaos_symphony_protocol::ShipSpawnEvent;
use tracing::instrument;

use crate::{
    authority::{ClientAuthority, ServerAuthority},
    identity::Identity,
    routing::Request,
    ship::{Ship, ShipBundle},
};

/// Ship Spawn Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ShipSpawnPlugin;

impl Plugin for ShipSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, event);
    }
}

#[instrument(skip_all)]
#[allow(clippy::needless_pass_by_value)]
fn event(
    mut commands: Commands,
    events: Query<(Entity, &Request<ShipSpawnEvent>)>,
    ships: Query<&Identity, With<Ship>>,
) {
    events.for_each(|(entity, event)| {
        commands.entity(entity).despawn();

        let message = &event.inner;

        let span = error_span!(
            "event",
            event_id = message.id,
            identity =? message.payload.identity.id
        );
        let _guard = span.enter();

        let message_payload_identity = message.payload.identity.clone().into();

        if ships
            .iter()
            .any(|identity| *identity == message_payload_identity)
        {
            debug!("already spawned");
            return;
        }

        info!(identity =? message.payload.identity.id, "spawned");
        commands.spawn(ShipBundle {
            ship: Ship,
            identity: message_payload_identity,
            client_authority: ClientAuthority::new(message.payload.client_authority.clone().into()),
            server_authority: ServerAuthority::new(message.payload.server_authority.clone().into()),
            transformation: message.payload.transformation.into(),
        });
    });
}
