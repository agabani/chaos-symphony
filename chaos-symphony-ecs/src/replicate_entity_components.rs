use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ReplicateEntityComponentsCallback, ReplicateEntityComponentsRequest,
    ReplicateEntityComponentsRequestPayload, ReplicateEntityComponentsResponsePayload,
    Request as _,
};

use crate::types::{EntityIdentity, NetworkIdentity, ReplicateSink};

/// Replicate Entity Components Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicateEntityComponentsPlugin;

impl Plugin for ReplicateEntityComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, initiate));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn callback(
    mut commands: Commands,
    callbacks: Query<(Entity, &ReplicateEntityComponentsCallback)>,
) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("callback", message_id =% callback.id);
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            let mut commands = commands.entity(entity);

            commands.remove::<ReplicateEntityComponentsCallback>();

            let Ok(response) = result else {
                error!("failed to receive response from server");
                return;
            };

            match response.payload {
                ReplicateEntityComponentsResponsePayload::Failure => {
                    error!("rejected by server");
                }
                ReplicateEntityComponentsResponsePayload::Success => {
                    info!("accepted by server");
                    commands.insert(ReplicateSink);
                }
            };
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::type_complexity)]
fn initiate(
    mut commands: Commands,
    identities: Query<
        (Entity, &EntityIdentity),
        (
            Without<ReplicateSink>,
            Without<ReplicateEntityComponentsCallback>,
        ),
    >,
    endpoints: Query<&NetworkEndpoint, With<NetworkIdentity>>,
) {
    if let Some(endpoint) = endpoints.iter().next() {
        identities.for_each(|(entity, identity)| {
            let request = ReplicateEntityComponentsRequest::message(
                Uuid::new_v4(),
                ReplicateEntityComponentsRequestPayload {
                    entity_identity: identity.inner.clone().into(),
                },
            );

            let Ok(callback) = request.try_send(endpoint) else {
                error!("failed to send request");
                return;
            };

            info!("request sent");
            commands.entity(entity).insert(callback);
        });
    }
}
