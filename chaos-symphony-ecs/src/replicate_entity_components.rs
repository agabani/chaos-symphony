use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ReplicateEntityComponentsCallback, ReplicateEntityComponentsRequest,
    ReplicateEntityComponentsRequestPayload, ReplicateEntityComponentsResponse,
    ReplicateEntityComponentsResponsePayload, Request as _, Response as _,
};

use crate::types::{
    EntityAuthority, EntityIdentity, EntityReplicationAuthority, EntitySimulationAuthority,
    NetworkIdentity, NetworkReplicationAuthority, NetworkSimulationAuthority, ReplicateSink,
    ReplicateSource, Role, Trusted, Untrusted,
};

/// Replicate Entity Components Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicateEntityComponentsPlugin {
    /// Role.
    role: Role,
}

impl ReplicateEntityComponentsPlugin {
    /// Creates a new [`ReplicateEntityComponentsPlugin`].
    #[must_use]
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

impl Plugin for ReplicateEntityComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Trusted<ReplicateEntityComponentsRequest>>()
            .add_event::<Untrusted<ReplicateEntityComponentsRequest>>()
            .add_systems(Update, callback)
            .add_systems(Update, request);

        match self.role {
            Role::Client | Role::Simulation => {
                app.add_systems(
                    Update,
                    initiate::<EntityReplicationAuthority, NetworkReplicationAuthority>,
                );
            }
            Role::Replication => {
                app.add_systems(
                    Update,
                    initiate::<EntitySimulationAuthority, NetworkSimulationAuthority>,
                )
                .add_systems(Update, validate_request);
            }
        }
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
fn initiate<EA, NA>(
    mut commands: Commands,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity), With<NA>>,
    entities: Query<
        (Entity, &EA, &EntityIdentity),
        (
            Without<ReplicateSink>,
            Without<ReplicateSource>,
            Without<ReplicateEntityComponentsCallback>,
        ),
    >,
) where
    EA: EntityAuthority + Component,
    NA: Component,
{
    entities.for_each(|(entity, entity_authority, entity_identity)| {
        let request_id = Uuid::new_v4();
        let span = error_span!("initiate", message_id =%  request_id);
        let _guard = span.enter();

        let Some((endpoint, _)) = endpoints
            .iter()
            .find(|(_, network_identity)| network_identity.inner == *entity_authority.identity())
        else {
            error!("network identity does not exist");
            return;
        };

        let request = ReplicateEntityComponentsRequest::message(
            request_id,
            ReplicateEntityComponentsRequestPayload {
                entity_identity: entity_identity.inner.clone().into(),
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

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut reader: EventReader<Trusted<ReplicateEntityComponentsRequest>>,
    endpoints: Query<&NetworkEndpoint>,
) {
    reader.read().for_each(|request| {
        let span = error_span!("request", message_id =%  request.inner.id);
        let _guard = span.enter();

        let Some(source_endpoint_id) = &request.inner.header.source_endpoint_id else {
            error!("request does not have source network endpoint");
            return;
        };

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == *source_endpoint_id)
        else {
            error!("network endpoint does not exist");
            return;
        };

        let response = ReplicateEntityComponentsResponse::message(
            request.inner.id,
            ReplicateEntityComponentsResponsePayload::Success,
        );

        if response.try_send(endpoint).is_err() {
            error!("failed to send response");
            return;
        };
        info!("response sent");
    });
}

fn validate_request(
    mut reader: EventReader<Untrusted<ReplicateEntityComponentsRequest>>,
    mut writer: EventWriter<Trusted<ReplicateEntityComponentsRequest>>,
) {
    reader.read().for_each(|request| {
        let span = error_span!("request", message_id =%  request.inner.id);
        let _guard = span.enter();

        // TODO: validate requesters permissions to entity.

        writer.send(Trusted {
            inner: request.inner.clone(),
        });
    });
}
