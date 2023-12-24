use bevy::prelude::*;
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkIdentity, NetworkMessage},
    ship::Ship,
    transform::Transformation,
    types::{EntityClientAuthority, EntityServerAuthority, Identity},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{ReplicateRequest, ReplicateResponse, ReplicateResponsePayload};

use crate::types::Replicate;

/// Replicate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicatePlugin;

impl Plugin for ReplicatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (initiate, request));
    }
}

fn initiate(
    mut commands: Commands,
    identities: Query<(&Identity, &EntityServerAuthority), Added<Identity>>,
) {
    identities.for_each(|identity| {
        error!(identity=?identity,"added");
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
