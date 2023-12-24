use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    IdentitiesCallback, IdentitiesRequest, IdentitiesRequestPayload, IdentitiesResponsePayload,
    Request as _,
};

use crate::types::NetworkIdentity;

/// Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct IdentitiesPlugin;

impl Plugin for IdentitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, initiate));
    }
}

#[derive(Debug, Clone, Copy, Component, Reflect)]
struct Identities;

#[tracing::instrument(skip_all)]
fn callback(mut commands: Commands, callbacks: Query<(Entity, &IdentitiesCallback)>) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("callback", message_id =% callback.id);
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            let mut commands = commands.entity(entity);

            commands.remove::<IdentitiesCallback>();

            let Ok(response) = result else {
                error!("failed to receive response from server");
                return;
            };

            match response.payload {
                IdentitiesResponsePayload::Failure => {
                    error!("rejected by server");
                }
                IdentitiesResponsePayload::Success => {
                    info!("accepted by server");
                    commands.insert(Identities);
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
            Without<Identities>,
            Without<IdentitiesCallback>,
        ),
    >,
) {
    endpoints.for_each(|(entity, endpoint)| {
        let request = IdentitiesRequest::message(Uuid::new_v4(), IdentitiesRequestPayload {});

        let Ok(callback) = request.try_send(endpoint) else {
            error!("failed to send request");
            return;
        };

        info!("request sent");
        commands.entity(entity).insert(callback);
    });
}
