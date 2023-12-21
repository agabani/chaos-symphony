use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    entity::Identity,
    ship::{Ship, ShipBundle},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ShipSpawnRequest, ShipSpawning};
use tracing::instrument;

#[instrument(skip_all)]
pub fn callback(mut commands: Commands, ship_spawnings: Query<(Entity, &ShipSpawning)>) {
    ship_spawnings.for_each(|(entity, ship_spawning)| {
        let span = error_span!("callback", request_id = ship_spawning.id);
        let _guard = span.enter();

        if let Poll::Ready(result) = ship_spawning.try_poll() {
            commands.entity(entity).despawn();

            let Ok(response) = result else {
                error!("failed to receive response from server");
                return;
            };

            if !response.success {
                warn!("server rejected request");
                return;
            }

            info!(identity =? response.identity, "spawned");
            commands.spawn(ShipBundle {
                ship: Ship,
                identity: Identity::new(response.identity),
                client_authority: ClientAuthority::new(response.client_authority),
                server_authority: ServerAuthority::new(response.server_authority),
            });
        }
    });
}

#[instrument(skip_all)]
pub fn request(
    mut commands: Commands,
    endpoints: Query<&NetworkEndpoint, With<ClientAuthority>>,
    ships: Query<(), With<Ship>>,
    ship_spawning: Query<(), With<ShipSpawning>>,
) {
    if let Some(endpoint) = endpoints.iter().next() {
        let count = ships.iter().count() + ship_spawning.iter().count();

        for _ in count..1 {
            let id = Uuid::new_v4().to_string();

            let span = error_span!("request", request_id =? id);
            let _guard = span.enter();

            let Ok(ship_spawning) = ShipSpawnRequest::new(id).try_send(endpoint) else {
                error!("failed to send request to server");
                continue;
            };

            info!("sent request to server");
            commands.spawn(ship_spawning);
        }
    }
}
