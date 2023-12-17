use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_async::Poll;
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{AuthenticateRequest, Authenticating};

/// Network Authenticate Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct NetworkAuthenticatePlugin {
    /// Identity.
    pub identity: String,
}

impl Plugin for NetworkAuthenticatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Identity {
            inner: self.identity.clone(),
        })
        .add_systems(Update, (authenticate, authenticating));
    }
}

/// Identity.
#[derive(Resource)]
struct Identity {
    inner: String,
}

/// Authenticate.
///
/// Initiates authentication when a new [`NetworkEndpoint`] is created.
#[allow(clippy::needless_pass_by_value)]
fn authenticate(
    mut commands: Commands,
    identity: Res<Identity>,
    endpoints: Query<&NetworkEndpoint, Added<NetworkEndpoint>>,
) {
    endpoints.for_each(|endpoint| {
        let request = AuthenticateRequest {
            id: Uuid::new_v4().to_string(),
            identity: identity.inner.clone(),
        };

        match request.try_send(endpoint) {
            Ok(authenticating) => {
                commands.spawn(authenticating);
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
/// - On success, logs result.
/// - On ready, despawns [`Authenticating`].
#[allow(clippy::needless_pass_by_value)]
fn authenticating(mut commands: Commands, authenticatings: Query<(Entity, &Authenticating)>) {
    authenticatings.for_each(|(entity, authenticating)| {
        if let Poll::Ready(result) = authenticating.try_poll() {
            commands.entity(entity).despawn();

            let response = match result {
                Ok(result) => result,
                Err(error) => {
                    error!(error =? error, "failed to authenticate");
                    return;
                }
            };

            info!(
                id = response.id,
                success = response.success,
                "authenticating"
            );
        }
    });
}
