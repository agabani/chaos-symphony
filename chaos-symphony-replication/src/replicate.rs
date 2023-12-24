use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkIdentity, NetworkMessage},
    ship::Ship,
    transform::Transformation,
    types::{EntityClientAuthority, EntityServerAuthority, Identity},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ReplicateCallback, ReplicateRequest, ReplicateRequestPayload, ReplicateResponse,
    ReplicateResponsePayload,
};

use crate::types::Replicate;

/// Replicate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicatePlugin;

impl Plugin for ReplicatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, initiate, request));
    }
}

#[derive(Debug, Clone, Component)]
struct Replicated;

#[allow(clippy::needless_pass_by_value)]
fn callback(mut commands: Commands, callbacks: Query<(Entity, &ReplicateCallback)>) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("callback", message_id =% callback.id);
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            commands.entity(entity).remove::<ReplicateCallback>();

            let Ok(response) = result else {
                error!("failed to receive response from server");
                return;
            };

            let ReplicateResponsePayload::Success = response.payload else {
                warn!("server rejected request");
                return;
            };

            info!("server accepted request");
            commands.entity(entity).insert(Replicated);
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
fn initiate(
    mut commands: Commands,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity)>,
    identities: Query<(Entity, &Identity, &EntityServerAuthority), Added<Identity>>,
) {
    identities.for_each(|(entity, identity, entity_server_authority)| {
        error!(identity=?identity,"added");

        let endpoint = endpoints.iter().find(|(_, endpoint_identity)| {
            endpoint_identity.inner == entity_server_authority.identity
        });

        let Some((endpoint, _)) = endpoint else {
            error!("server authority endpoint not found, unable to initiate replication");
            return;
        };

        let request = ReplicateRequest::new(
            Uuid::new_v4(),
            ReplicateRequestPayload {
                identity: identity.clone().into(),
            },
        );

        let Ok(callback) = request.try_send(endpoint) else {
            error!("replicate request failed, unable to initiate replication");
            return;
        };

        commands.entity(entity).insert(callback);
    })
}

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut commands: Commands,
    messages: Query<(
        Entity,
        &NetworkEndpointId,
        &NetworkMessage<ReplicateRequest>,
    )>,
    endpoints: Query<&NetworkEndpoint, With<NetworkIdentity>>,
    identities: Query<&Identity>,
) {
    messages.for_each(|(entity, endpoint_id, message)| {
        let span = error_span!("request", message_id =% message.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let message = &message.inner;

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == endpoint_id.inner)
        else {
            warn!("endpoint not found");
            return;
        };

        let identity: Identity = message.payload.identity.clone().into();
        if !identities.iter().any(|i| *i == identity) {
            warn!("identity does not exist");

            let response = ReplicateResponse::new(message.id, ReplicateResponsePayload::Failure);
            if response.try_send(endpoint).is_err() {
                warn!("failed to send response");
                return;
            }

            return;
        }

        let response = ReplicateResponse::new(message.id, ReplicateResponsePayload::Success);
        if response.try_send(endpoint).is_err() {
            warn!("failed to send response");
            return;
        }

        info!("replicating");
        commands.spawn((
            *endpoint_id,
            Replicate::<EntityClientAuthority>::new(identity.clone()),
        ));
        commands.spawn((
            *endpoint_id,
            Replicate::<EntityServerAuthority>::new(identity.clone()),
        ));
        commands.spawn((
            //
            *endpoint_id,
            Replicate::<Ship>::new(identity.clone()),
        ));
        commands.spawn((
            *endpoint_id,
            Replicate::<Transformation>::new(identity.clone()),
        ));
    });
}
