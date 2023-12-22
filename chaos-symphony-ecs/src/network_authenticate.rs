use bevy::{
    prelude::*,
    utils::{tracing::instrument, Uuid},
};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{AuthenticateRequest, AuthenticateRequestPayload, Authenticating};

use crate::{
    authority::{ClientAuthority, ServerAuthority},
    identity::Identity,
};

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
            Uuid::new_v4().to_string(),
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
fn authenticating(mut commands: Commands, authenticatings: Query<(Entity, &Authenticating)>) {
    authenticatings.for_each(|(entity, authenticating)| {
        let span = error_span!("authenticating", id = authenticating.id());
        let _guard = span.enter();

        if let Poll::Ready(result) = authenticating.try_poll() {
            let response = match result {
                Ok(result) => {
                    commands.entity(entity).remove::<Authenticating>();
                    result
                }
                Err(error) => {
                    error!(error =? error, "failed to authenticate");
                    commands.entity(entity).despawn();
                    return;
                }
            };

            if !response.payload.success {
                error!("failed to authenticate");
                commands.entity(entity).despawn();
                return;
            }

            match response.payload.identity.noun.as_str() {
                "ai" | "client" => {
                    let authority = ClientAuthority::new(response.payload.identity.into());
                    info!(authority =? authority, "authenticated");
                    commands.entity(entity).insert(authority);
                }
                "simulation" => {
                    let authority = ServerAuthority::new(response.payload.identity.into());
                    info!(authority =? authority, "authenticated");
                    commands.entity(entity).insert(authority);
                }
                identity => todo!("{identity}"),
            };
        }
    });
}
