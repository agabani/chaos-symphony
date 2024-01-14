use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    EntityIdentitiesCallback, EntityIdentitiesRequest, EntityIdentitiesRequestPayload,
    EntityIdentitiesResponse, EntityIdentitiesResponsePayload, EntityIdentityEvent,
    EntityIdentityEventPayload, Event as _, Request as _, Response as _,
};

use crate::types::{EntityIdentity, NetworkIdentity, Role, Trusted, Untrusted};

/// Entity Identities Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct EntityIdentitiesPlugin {
    role: Role,
}

impl EntityIdentitiesPlugin {
    /// Creates a new [`EntityIdentitiesPlugin`].
    #[must_use]
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

impl Plugin for EntityIdentitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Trusted<EntityIdentitiesRequest>>()
            .add_event::<Untrusted<EntityIdentitiesRequest>>();

        match self.role {
            Role::Client | Role::Simulation => {
                app.add_systems(Update, (callback, initiate));
            }
            Role::Replication => {
                app.add_systems(Update, request);
            }
        }
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

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut reader: EventReader<Untrusted<EntityIdentitiesRequest>>,
    endpoints: Query<&NetworkEndpoint>,
    entity_identities: Query<&EntityIdentity>,
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

        entity_identities.for_each(|entity_identity| {
            // TODO: filter entity identities using requesters permissions.

            let request = EntityIdentityEvent::message(
                Uuid::new_v4(),
                EntityIdentityEventPayload {
                    inner: entity_identity.inner.clone().into(),
                },
            );

            if request.try_send(endpoint).is_err() {
                warn!("failed to send event");
            }
        });
    });
}
