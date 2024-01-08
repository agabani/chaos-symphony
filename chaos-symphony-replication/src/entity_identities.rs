use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::types::{EntityIdentity, Untrusted};
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
        app.add_event::<Untrusted<EntityIdentitiesRequest>>()
            .add_systems(Update, request);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut reader: EventReader<Untrusted<EntityIdentitiesRequest>>,
    endpoints: Query<&NetworkEndpoint>,
    identities: Query<&EntityIdentity>,
) {
    reader.read().for_each(|request| {
        let span = error_span!("request", message_id =% request.inner.id);
        let _guard = span.enter();

        let Some(source_endpoint_id) = &request.inner.header.source_endpoint_id else {
            error!("request does not have source endpoint id");
            return;
        };

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == *source_endpoint_id)
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
