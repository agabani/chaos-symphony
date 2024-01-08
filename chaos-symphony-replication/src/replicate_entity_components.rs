use bevy::prelude::*;

use chaos_symphony_ecs::types::{Trusted, Untrusted};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ReplicateEntityComponentsRequest, ReplicateEntityComponentsResponse,
    ReplicateEntityComponentsResponsePayload, Response as _,
};

/// Replicate Entity Components Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ReplicateEntityComponentsPlugin;

impl Plugin for ReplicateEntityComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, request);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut reader: EventReader<Untrusted<ReplicateEntityComponentsRequest>>,
    mut writer: EventWriter<Trusted<ReplicateEntityComponentsRequest>>,
    endpoints: Query<&NetworkEndpoint>,
) {
    reader.read().for_each(|request| {
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
            error!("failed to send event");
            return;
        };

        writer.send(Trusted {
            inner: request.inner.clone(),
        });
    });
}
