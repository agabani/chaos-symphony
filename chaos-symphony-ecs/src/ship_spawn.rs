use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
};
use chaos_symphony_protocol::ShipSpawnEvent;
use tracing::instrument;

use crate::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    routing::Request,
    ship::{Ship, ShipBundle},
    transform::Transformation,
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
            transformation: Transformation {
                orientation: DQuat {
                    x: event.orientation_x,
                    y: event.orientation_y,
                    z: event.orientation_z,
                    w: event.orientation_w,
                },
                position: DVec3 {
                    x: event.position_x,
                    y: event.position_y,
                    z: event.position_z,
                },
            },
        });
    });
}
