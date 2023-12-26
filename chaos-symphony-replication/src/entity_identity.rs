use bevy::{prelude::*, utils::Uuid};
use chaos_symphony_ecs::{
    network::NetworkMessage,
    types::{EntityIdentity, NetworkIdentity, ReplicateSource},
};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{EntityIdentityEvent, EntityIdentityEventPayload, Event};

/// Entity Identity Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct EntityIdentityPlugin;

impl Plugin for EntityIdentityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (event, replicate));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn event(mut commands: Commands, messages: Query<(Entity, &NetworkMessage<EntityIdentityEvent>)>) {
    messages.for_each(|(entity, message)| {
        let message = &message.inner;
        commands.entity(entity).despawn();

        let span = error_span!("event", message_id =% message.id);
        let _guard = span.enter();

        let payload = &message.payload;

        let entity_identity = EntityIdentity {
            inner: payload.inner.clone().into(),
        };
        info!(entity_identity =? entity_identity, "spawned");
        commands.spawn((entity_identity, ReplicateSource));
    });
}

#[allow(clippy::needless_pass_by_value)]
fn replicate(
    entity_identities: Query<&EntityIdentity, (Changed<EntityIdentity>, With<ReplicateSource>)>,
    endpoints: Query<&NetworkEndpoint, With<NetworkIdentity>>,
) {
    entity_identities.for_each(|entity_identity| {
        endpoints.for_each(|endpoint| {
            let message = EntityIdentityEvent::message(
                Uuid::new_v4(),
                EntityIdentityEventPayload {
                    inner: entity_identity.inner.clone().into(),
                },
            );

            if message.try_send(endpoint).is_err() {
                error!("failed to send message");
            }
        });
    });
}
