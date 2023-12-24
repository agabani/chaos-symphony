use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{network::NetworkIdentity, ship::Ship, types::Identity};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ShipSpawnRequest, ShipSpawnRequestPayload, ShipSpawnResponsePayload, ShipSpawning,
};

pub struct ShipSpawnPlugin;

impl Plugin for ShipSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, initiate));
    }
}

fn callback(mut commands: Commands, callbacks: Query<(Entity, &ShipSpawning)>) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("callback", message_id =% callback.id);
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            commands.entity(entity).despawn();

            let Ok(response) = result else {
                error!("failed to receive response from server");
                return;
            };

            let ShipSpawnResponsePayload::Success(success) = response.payload else {
                warn!("server rejected request");
                return;
            };

            info!(identity =% success.identity, "spawned");
            let identity: Identity = success.identity.into();
            commands.spawn((identity, Ship));
        }
    });
}

fn initiate(
    mut commands: Commands,
    endpoints: Query<&NetworkEndpoint, With<NetworkIdentity>>,
    ships: Query<(), With<Ship>>,
    callbacks: Query<(), With<ShipSpawning>>,
) {
    if let Some(endpoint) = endpoints.iter().next() {
        let count = ships.iter().count() + callbacks.iter().count();

        for _ in count..1 {
            let id = Uuid::new_v4();

            let span = error_span!("request", message_id =% id);
            let _guard = span.enter();

            let Ok(callback) = ShipSpawnRequest::new(
                id,
                ShipSpawnRequestPayload {
                    client_identity: None,
                    server_identity: None,
                },
            )
            .try_send(endpoint) else {
                error!("failed to send request to server");
                continue;
            };

            info!("sent request to server");
            commands.spawn(callback);
        }
    }
}
