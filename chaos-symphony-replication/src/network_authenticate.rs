use bevy::prelude::*;
use chaos_symphony_ecs::types::{NetworkIdentity, Untrusted};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, AuthenticateResponse, AuthenticateResponsePayload, Response as _,
};

/// Network Authenticate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkAuthenticatePlugin {
    /// Identity.
    pub identity: NetworkIdentity,
}

impl Plugin for NetworkAuthenticatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Untrusted<AuthenticateRequest>>()
            .insert_resource(self.identity.clone())
            .add_systems(Update, request);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut commands: Commands,
    identity: Res<NetworkIdentity>,
    mut reader: EventReader<Untrusted<AuthenticateRequest>>,
    endpoints: Query<(Entity, &NetworkEndpoint)>,
) {
    reader.read().for_each(|request| {
        let span = error_span!("request", message_id =% request.inner.id);
        let _guard = span.enter();

        let Some(source_endpoint_id) = &request.inner.header.source_endpoint_id else {
            error!("request does not have source endpoint id");
            return;
        };

        let Some((entity, endpoint)) = endpoints
            .iter()
            .find(|(_, endpoint)| endpoint.id() == *source_endpoint_id)
        else {
            warn!("endpoint not found");
            return;
        };

        let mut commands = commands.entity(entity);
        let payload = &request.inner.payload;

        let network_identity = NetworkIdentity {
            inner: payload.identity.clone().into(),
        };
        info!(network_identity =? network_identity, "authenticated");
        commands.insert(network_identity);

        let response = AuthenticateResponse::message(
            request.inner.id,
            AuthenticateResponsePayload::Success {
                client_identity: payload.identity.clone(),
                server_identity: identity.inner.clone().into(),
            },
        );
        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
            warn!(error =? error, "failed to send response to endpoint");
        }
    });
}
