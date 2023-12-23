use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    IdentitiesCallback, IdentitiesEvent, IdentitiesRequest, IdentitiesRequestPayload,
    IdentitiesResponsePayload,
};

use crate::{
    authority::{ClientAuthority, ServerAuthority},
    identity::Identity,
    network::NetworkMessage,
};

/// Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct IdentitiesPlugin;

impl Plugin for IdentitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, event, request));
    }
}

fn callback(mut commands: Commands, callbacks: Query<(Entity, &IdentitiesCallback)>) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("callback", message_id =% callback.id);
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            commands.entity(entity).despawn();

            let Ok(response) = result else {
                error!("failed to receive response from server");
                return;
            };

            let IdentitiesResponsePayload::Success = response.payload else {
                warn!("server rejected request");
                return;
            };

            info!("server accepted request");
        }
    });
}

fn event(
    mut commands: Commands,
    messages: Query<(Entity, &NetworkMessage<IdentitiesEvent>)>,
    identities: Query<&Identity>,
) {
    messages.for_each(|(entity, message)| {
        commands.entity(entity).despawn();

        let message = &message.inner;

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let identity: Identity = message.payload.identity.clone().into();

        if identities.iter().any(|i| *i == identity) {
            debug!("already spawned");
            return;
        }

        info!(identity =% message.payload.identity, "spawned");
        commands.spawn(identity);
    });
}

fn request(
    mut commands: Commands,
    client_endpoints: Query<&NetworkEndpoint, Added<ClientAuthority>>,
    server_endpoints: Query<&NetworkEndpoint, Added<ServerAuthority>>,
) {
    let endpoints = server_endpoints.iter().chain(client_endpoints.iter());

    endpoints.for_each(|endpoint| {
        let span = error_span!("request", endpoint_id = endpoint.id());
        let _guard = span.enter();

        let request = IdentitiesRequest::new(Uuid::new_v4(), IdentitiesRequestPayload {});

        match request.try_send(endpoint) {
            Ok(callback) => {
                info!("sent request");
                commands.spawn(callback);
            }
            Err(error) => {
                error!(error =? error, "unable to send request");
            }
        }
    });
}
