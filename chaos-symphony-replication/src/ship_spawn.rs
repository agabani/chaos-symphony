use bevy::prelude::*;
use chaos_symphony_async::Poll;
use chaos_symphony_ecs::{
    network::{
        NetworkClientAuthority, NetworkEndpointId, NetworkIdentity, NetworkMessage,
        NetworkServerAuthority,
    },
    types::{EntityClientAuthority, EntityServerAuthority, Identity},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    ShipSpawnRequest, ShipSpawnResponse, ShipSpawnResponsePayload, ShipSpawning,
};

use crate::types::NetworkPath;

/// Ship Spawn Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ShipSpawnPlugin;

impl Plugin for ShipSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (callback, request));
    }
}

fn callback(
    mut commands: Commands,
    endpoints: Query<&NetworkEndpoint>,
    callbacks: Query<(
        Entity,
        &ShipSpawning,
        &NetworkPath,
        &EntityClientAuthority,
        &EntityServerAuthority,
    )>,
) {
    callbacks.for_each(
        |(entity, callback, network_path, client_authority, server_authority)| {
            let message_id = callback.id;
            if let Poll::Ready(result) = callback.try_poll() {
                let span = error_span!("callback", message_id =% message_id);
                let _guard = span.enter();

                commands.entity(entity).despawn();

                let client = endpoints
                    .iter()
                    .find(|endpoint| endpoint.id() == network_path.source.inner);

                let Ok(response) = result else {
                    error!("failed to receive response from server");

                    if let Some(endpoint) = &client {
                        let response =
                            ShipSpawnResponse::new(message_id, ShipSpawnResponsePayload::Failure);
                        if response.try_send(endpoint).is_err() {
                            warn!("failed to send failure to source");
                        }
                    }

                    return;
                };

                let ShipSpawnResponsePayload::Success(payload) = &response.payload else {
                    error!("server rejected request");
                    if let Some(endpoint) = &client {
                        let response =
                            ShipSpawnResponse::new(message_id, ShipSpawnResponsePayload::Failure);
                        if response.try_send(endpoint).is_err() {
                            warn!("failed to send failure to source");
                        }
                    };
                    return;
                };

                let identity: Identity = payload.identity.clone().into();
                info!(identity =? identity, "spawned");
                commands.spawn((identity, client_authority.clone(), server_authority.clone()));

                if let Some(endpoint) = client {
                    if response.try_send(endpoint).is_err() {
                        warn!("failed to send success to source");
                    }
                }
            };
        },
    );
}

fn request(
    mut commands: Commands,
    messages: Query<(
        Entity,
        &NetworkEndpointId,
        &NetworkMessage<ShipSpawnRequest>,
    )>,
    clients: Query<(&NetworkEndpoint, &NetworkIdentity), With<NetworkClientAuthority>>,
    servers: Query<(&NetworkEndpoint, &NetworkIdentity), With<NetworkServerAuthority>>,
) {
    messages.for_each(|(entity, endpoint_id, message)| {
        let span = error_span!("request", message_id =% message.inner.id);
        let _guard = span.enter();

        commands.entity(entity).despawn();

        let Some((client_endpoint, client_identity)) = clients
            .iter()
            .find(|(endpoint, _)| endpoint.id() == endpoint_id.inner)
        else {
            warn!("client endpoint not found");
            return;
        };

        let request = &message.inner;

        let Some((server_endpoint, server_identity)) = servers.iter().next() else {
            warn!("server unavailable");
            let response = ShipSpawnResponse::new(request.id, ShipSpawnResponsePayload::Failure);
            if response.try_send(client_endpoint).is_err() {
                warn!("failed to send response to client");
            }
            return;
        };

        let id = request.id;
        let mut request = request.clone();
        // overwriting metadata.
        request.payload.client_identity = Some(client_identity.inner.clone().into());
        request.payload.server_identity = Some(server_identity.inner.clone().into());

        let Ok(callback) = request.try_send(server_endpoint) else {
            error!("failed to send request to server");
            let response = ShipSpawnResponse::new(id, ShipSpawnResponsePayload::Failure);
            if response.try_send(client_endpoint).is_err() {
                warn!("failed to send response to client");
            }
            return;
        };

        info!("sent request to server");
        commands.spawn((
            callback,
            NetworkPath {
                source: NetworkEndpointId {
                    inner: client_endpoint.id(),
                },
                target: NetworkEndpointId {
                    inner: server_endpoint.id(),
                },
            },
            EntityClientAuthority {
                identity: client_identity.inner.clone(),
            },
            EntityServerAuthority {
                identity: server_identity.inner.clone(),
            },
        ));
    });
}
