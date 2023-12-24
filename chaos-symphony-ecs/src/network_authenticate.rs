use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateCallback, AuthenticateRequest, AuthenticateRequestPayload,
    AuthenticateResponsePayload, Request as _,
};

use crate::types::NetworkIdentity;

/// Network Authenticate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkAuthenticatePlugin {
    /// Identity.
    pub identity: NetworkIdentity,
}

impl Plugin for NetworkAuthenticatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.identity.clone())
            .add_systems(Update, (request, callback));
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

            let AuthenticateResponsePayload::Success { identity } = response.payload else {
                error!("failed to authenticate");
                commands.despawn();
                return;
            };

            let network_identity = NetworkIdentity {
                inner: identity.into(),
            };
            info!(network_identity =? network_identity, "authenticated");
            commands.insert(network_identity);
        }
    });
}

/// Request.
///
/// Initiates authentication when a new [`NetworkEndpoint`] is created.
#[allow(clippy::needless_pass_by_value)]
fn request(
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
