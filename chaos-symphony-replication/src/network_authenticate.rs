use bevy::prelude::*;
use chaos_symphony_ecs::{
    network::{NetworkEndpointId, NetworkMessage},
    types::{NetworkClientAuthority, NetworkIdentity, NetworkServerAuthority},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, AuthenticateResponse, AuthenticateResponsePayload, Response as _,
};

/// Network Authenticate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkAuthenticatePlugin;

impl Plugin for NetworkAuthenticatePlugin {
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

        let identity = &message.inner.payload.identity;

        match identity.noun.as_str() {
            "ai" | "client" => {
                commands.insert(NetworkClientAuthority);
            }
            "simulation" => {
                commands.insert(NetworkServerAuthority);
            }
            noun => todo!("{noun}"),
        };

        let network_identity = NetworkIdentity {
            inner: identity.clone().into(),
        };
        info!(network_identity =? network_identity, "authenticated");
        commands.insert(network_identity);

        let response = AuthenticateResponse::message(
            message.inner.id,
            AuthenticateResponsePayload::Success {
                identity: identity.clone(),
            },
        );
        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
            warn!(error =? error, "failed to send response to endpoint");
        }
    });
}
