use bevy::{
    prelude::*,
    utils::{tracing::instrument, Uuid},
};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    AuthenticateCallback, AuthenticateRequest, AuthenticateRequestPayload,
    AuthenticateResponsePayload,
};

use crate::types::{ClientAuthority, Identity, ServerAuthority};

/// Network Authenticate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkAuthenticatePlugin {
    /// Identity.
    pub identity: Identity,
}

impl Plugin for NetworkAuthenticatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkIdentity {
            inner: self.identity.clone(),
        })
        .add_systems(Update, (authenticate, authenticating));
    }
}

/// Identity.
#[derive(Resource)]
struct NetworkIdentity {
    /// Inner.
    inner: Identity,
}

/// Authenticate.
///
/// Initiates authentication when a new [`NetworkEndpoint`] is created.
#[allow(clippy::needless_pass_by_value)]
fn authenticate(
    mut commands: Commands,
    identity: Res<NetworkIdentity>,
    endpoints: Query<(Entity, &NetworkEndpoint), Added<NetworkEndpoint>>,
) {
    endpoints.for_each(|(entity, endpoint)| {
        let request = AuthenticateRequest::new(
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

/// Authenticating.
///
/// Manages [`Authenticating`] lifetime.
/// - On ready, removes [`Authenticating`].
/// - On error, despawns entity.
/// - On failure, despawns entity.
/// - On success, inserts authority.
#[allow(clippy::needless_pass_by_value)]
#[instrument(skip_all)]
fn authenticating(mut commands: Commands, callbacks: Query<(Entity, &AuthenticateCallback)>) {
    callbacks.for_each(|(entity, callback)| {
        let span = error_span!("authenticating", message_id =% callback.id());
        let _guard = span.enter();

        if let Poll::Ready(result) = callback.try_poll() {
            let response = match result {
                Ok(result) => {
                    commands.entity(entity).remove::<AuthenticateCallback>();
                    result
                }
                Err(error) => {
                    error!(error =? error, "failed to authenticate");
                    commands.entity(entity).despawn();
                    return;
                }
            };

            let span = error_span!("authenticating", message_id =% response.id);
            let _guard = span.enter();

            let AuthenticateResponsePayload::Success { identity } = response.payload else {
                error!("failed to authenticate");
                commands.entity(entity).despawn();
                return;
            };

            match identity.noun.as_str() {
                "ai" | "client" => {
                    let authority = ClientAuthority::new(identity.into());
                    info!(authority =? authority, "authenticated");
                    commands.entity(entity).insert(authority);
                }
                "simulation" => {
                    let authority = ServerAuthority::new(identity.into());
                    info!(authority =? authority, "authenticated");
                    commands.entity(entity).insert(authority);
                }
                identity => todo!("{identity}"),
            };
        }
    });
}
