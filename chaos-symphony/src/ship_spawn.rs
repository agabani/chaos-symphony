use bevy::prelude::*;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    routing::Request,
    ship::{Ship, ShipBundle},
};
use chaos_symphony_protocol::ShipSpawnEvent;

#[allow(clippy::needless_pass_by_value)]
pub fn event(
    mut commands: Commands,
    events: Query<(Entity, &Request<ShipSpawnEvent>)>,
    ships: Query<&Identity, With<Ship>>,
) {
    events.for_each(|(entity, event)| {
        commands.entity(entity).despawn();

        let event = &event.inner;

        let span = error_span!("event", event_id = event.id, identity = event.identity);
        let _guard = span.enter();

        if ships.iter().any(|identity| identity.id() == event.identity) {
            debug!("already spawned");
            return;
        }

        info!(identity =? event.identity, "spawned");
        commands.spawn(ShipBundle {
            ship: Ship,
            identity: Identity::new(event.identity.clone()),
            client_authority: ClientAuthority::new(event.client_authority.clone()),
            server_authority: ServerAuthority::new(event.server_authority.clone()),
        });
    });
}
