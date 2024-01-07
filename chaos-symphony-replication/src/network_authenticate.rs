use bevy::prelude::*;
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    types::NetworkIdentity,
};
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
        app.insert_resource(self.identity.clone())
            .add_systems(Update, request);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn request(
    mut commands: Commands,
    identity: Res<NetworkIdentity>,
    messages: Query<(
        Entity,
        &NetworkEndpointId,
        &NetworkMessage<AuthenticateRequest>,
    )>,
    endpoints: Query<(Entity, &NetworkEndpoint)>,
) {
    messages.for_each(|(entity, endpoint_id, message)| {
        let span = error_span!("request", message_id =% message.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some((entity, endpoint)) = endpoints
            .iter()
            .find(|(_, endpoint)| endpoint.id() == endpoint_id.inner)
        else {
            warn!("endpoint not found");
            return;
        };

        let mut commands = commands.entity(entity);
        let payload = &message.inner.payload;

        let network_identity = NetworkIdentity {
            inner: payload.identity.clone().into(),
        };
        info!(network_identity =? network_identity, "authenticated");
        commands.insert(network_identity);

        let response = AuthenticateResponse::message(
            message.inner.id,
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
