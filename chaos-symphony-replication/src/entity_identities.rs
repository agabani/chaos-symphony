use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    types::{EntityIdentity, NetworkIdentity},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    EntityIdentitiesRequest, EntityIdentitiesResponse, EntityIdentitiesResponsePayload,
    EntityIdentityEvent, EntityIdentityEventPayload, Event, Response,
};

/// Entity Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct EntityIdentitiesPlugin;

impl Plugin for EntityIdentitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, request);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut commands: Commands,
    messages: Query<(
        Entity,
        &NetworkEndpointId,
        &NetworkMessage<EntityIdentitiesRequest>,
    )>,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity)>,
    identities: Query<&EntityIdentity>,
) {
    messages.for_each(|(entity, endpoint_id, request)| {
        let span = error_span!("request", message_id =% request.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some((endpoint, _identity)) = endpoints
            .iter()
            .find(|(endpoint, _)| endpoint.id() == endpoint_id.inner)
        else {
            warn!("endpoint not found");
            return;
        };

        let response = EntityIdentitiesResponse::message(
            request.inner.id,
            EntityIdentitiesResponsePayload::Success,
        );

        if response.try_send(endpoint).is_err() {
            warn!("failed to send response");
        }

        info!("sent response");

        identities.for_each(|identity| {
            let request = EntityIdentityEvent::message(
                Uuid::new_v4(),
                EntityIdentityEventPayload {
                    inner: identity.inner.clone().into(),
                },
            );

            if request.try_send(endpoint).is_err() {
                warn!("failed to send event");
            }
        });
    });
}
