use bevy::prelude::*;
use chaos_symphony_ecs::{
    authority::{ClientAuthority, ServerAuthority},
    routing::{EndpointId, Request},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{AuthenticateRequest, AuthenticateResponse};
use tracing::instrument;

#[instrument(skip_all)]
pub fn request(
    mut commands: Commands,
    requests: Query<(Entity, &EndpointId, &Request<AuthenticateRequest>)>,
    endpoints: Query<(Entity, &NetworkEndpoint)>,
) {
    requests.for_each(|(entity, endpoint_id, request)| {
        let span = error_span!("request", request_id = request.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some((entity, endpoint)) = endpoints
            .iter()
            .find(|(_, endpoint)| endpoint.id() == endpoint_id.inner)
        else {
            warn!("endpoint not found");
            return;
        };

        let identity = request.inner.identity.clone();

        match identity.as_str() {
            "ai" | "client" => {
                let authority = ClientAuthority::new(identity.clone());
                info!(authority =? authority, "authenticated");
                commands.entity(entity).insert(authority);
            }
            "simulation" => {
                let authority = ServerAuthority::new(identity.clone());
                info!(authority =? authority, "authenticated");
                commands.entity(entity).insert(authority);
            }
            identity => todo!("{identity}"),
        };

        let response = AuthenticateResponse {
            id: request.inner.id.clone(),
            success: true,
            identity,
        };

        if let Err(error) = endpoint.try_send_non_blocking(response.into()) {
            warn!(error =? error, "failed to send response to endpoint");
        }
    });
}
