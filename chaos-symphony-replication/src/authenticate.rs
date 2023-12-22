use bevy::prelude::*;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    network::{NetworkEndpointId, NetworkMessage},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateRequest, AuthenticateResponse, AuthenticateResponsePayload,
};
use tracing::instrument;

#[instrument(skip_all)]
pub fn request(
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

        let identity = &message.inner.payload.identity;

        match identity.noun.as_str() {
            "ai" | "client" => {
                let authority = ClientAuthority::new(identity.clone().into());
                info!(authority =? authority, "authenticated");
                commands.entity(entity).insert(authority);
            }
            "simulation" => {
                let authority = ServerAuthority::new(identity.clone().into());
                info!(authority =? authority, "authenticated");
                commands.entity(entity).insert(authority);
            }
            noun => todo!("{noun}"),
        };

        let response = AuthenticateResponse::new(
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
