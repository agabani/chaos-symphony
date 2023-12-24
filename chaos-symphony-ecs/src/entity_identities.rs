use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    EntityIdentitiesCallback, EntityIdentitiesRequest, EntityIdentitiesRequestPayload,
    EntityIdentitiesResponsePayload, Request as _,
};

use crate::types::NetworkIdentity;

/// Entity Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct EntityIdentitiesPlugin;

impl Plugin for EntityIdentitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, initiate));
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect)]
struct EntityIdentities;

#[tracing::instrument(skip_all)]
fn callback(mut commands: Commands, callbacks: Query<(Entity, &EntityIdentitiesCallback)>) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("callback", message_id =% callback.id);
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            let mut commands = commands.entity(entity);

            commands.remove::<EntityIdentitiesCallback>();

            let Ok(response) = result else {
                error!("failed to receive response from server");
                return;
            };

            match response.payload {
                EntityIdentitiesResponsePayload::Failure => {
                    error!("rejected by server");
                }
                EntityIdentitiesResponsePayload::Success => {
                    info!("accepted by server");
                    commands.insert(EntityIdentities);
                }
            };
        }
    });
}

#[allow(clippy::type_complexity)]
#[tracing::instrument(skip_all)]
fn initiate(
    mut commands: Commands,
    endpoints: Query<
        (Entity, &NetworkEndpoint),
        (
            With<NetworkIdentity>,
            Without<EntityIdentities>,
            Without<EntityIdentitiesCallback>,
        ),
    >,
) {
    endpoints.for_each(|(entity, endpoint)| {
        let request =
            EntityIdentitiesRequest::message(Uuid::new_v4(), EntityIdentitiesRequestPayload {});

        let Ok(callback) = request.try_send(endpoint) else {
            error!("failed to send request");
            return;
        };

        info!("request sent");
        commands.entity(entity).insert(callback);
    });
}
