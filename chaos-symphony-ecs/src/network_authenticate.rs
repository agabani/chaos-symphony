use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateCallback, AuthenticateRequest, AuthenticateRequestPayload, AuthenticateResponse,
    AuthenticateResponsePayload, Request as _, Response as _,
};

use crate::types::{NetworkIdentity, Role, Untrusted};

/// Network Authenticate Plugin.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct NetworkAuthenticatePlugin {
    /// Identity.
    pub identity: NetworkIdentity,

    /// Role.
    pub role: Role,
}

impl NetworkAuthenticatePlugin {
    /// Creates a new [`NetworkAuthenticatePlugin`].
    #[must_use]
    pub fn new(identity: NetworkIdentity, role: Role) -> Self {
        Self { identity, role }
    }
}

impl Plugin for NetworkAuthenticatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.identity.clone())
            .add_event::<Untrusted<AuthenticateRequest>>();

        match self.role {
            Role::Client | Role::Simulation => {
                app.add_systems(Update, (initiate, callback));
            }
            Role::Replication => {
                app.add_systems(Update, request);
            }
        }
    }
}

/// Callback.
///
/// Manages [`Authenticating`] lifetime.
/// - On ready, removes [`Authenticating`].
/// - On error, despawns entity.
/// - On failure, despawns entity.
/// - On success, inserts authority.
#[allow(clippy::needless_pass_by_value)]
fn callback(mut commands: Commands, callbacks: Query<(Entity, &AuthenticateCallback)>) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("callback", message_id =% callback.id());
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            let mut commands = commands.entity(entity);

            let response = match result {
                Ok(result) => {
                    commands.remove::<AuthenticateCallback>();
                    result
                }
                Err(error) => {
                    error!(error =? error, "failed to authenticate");
                    commands.despawn();
                    return;
                }
            };

            let AuthenticateResponsePayload::Success {
                client_identity,
                server_identity,
            } = response.payload
            else {
                error!("failed to authenticate");
                commands.despawn();
                return;
            };

            info!(
                client_identity =% client_identity,
                server_identity =% server_identity,
                "authenticated"
            );

            let network_identity = NetworkIdentity {
                inner: server_identity.into(),
            };
            commands.insert(network_identity);
        }
    });
}

/// Request.
///
/// Initiates authentication when a new [`NetworkEndpoint`] is created.
#[allow(clippy::needless_pass_by_value)]
fn initiate(
    mut commands: Commands,
    identity: Res<NetworkIdentity>,
    endpoints: Query<(Entity, &NetworkEndpoint), Added<NetworkEndpoint>>,
) {
    endpoints.for_each(|(entity, endpoint)| {
        let request = AuthenticateRequest::message(
            Uuid::new_v4(),
            AuthenticateRequestPayload {
                identity: identity.inner.clone().into(),
            },
        );

        match request.try_send(endpoint) {
            Ok(authenticating) => {
                commands.entity(entity).insert(authenticating);
            }
            Err(error) => {
                warn!(error =? error, "unable to send authenticate request");
            }
        }
    });
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
