use bevy::prelude::*;
use chaos_symphony_ecs::network::{
    NetworkClientAuthority, NetworkEndpointId, NetworkIdentity, NetworkMessage,
    NetworkServerAuthority,
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, AuthenticateResponse, AuthenticateResponsePayload,
};
use tracing::instrument;

/// Authenticate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct AuthenticatePlugin;

impl Plugin for AuthenticatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, request);
    }
}

#[instrument(skip_all)]
fn request(
    mut commands: Commands,
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

        let message = &message.inner;

        let network_identity = NetworkIdentity {
            inner: message.payload.identity.clone().into(),
        };
        info!(network_identity =? network_identity, "authenticated");

        match network_identity.inner.noun() {
            "ai" | "client" => {
                commands
                    .entity(entity)
                    .insert((network_identity, NetworkClientAuthority));
            }
            "simulation" => {
                commands
                    .entity(entity)
                    .insert((network_identity, NetworkServerAuthority));
            }
            noun => todo!("{noun}"),
        };

        let response = AuthenticateResponse::new(
            message.id,
            AuthenticateResponsePayload::Success {
                identity: message.payload.identity.clone(),
            },
        );
        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
            warn!(error =? error, "failed to send response to endpoint");
        }
    });
}
