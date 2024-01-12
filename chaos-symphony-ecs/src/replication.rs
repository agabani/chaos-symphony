use std::marker::PhantomData;

use bevy::{ecs::system::EntityCommands, prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::{
    Event as _, ReplicateEntityComponentsRequest, TransformationEvent, TransformationEventPayload,
};

use crate::types::{
    EntityAuthority, EntityIdentity, EntityReplicationAuthority, EntityServerAuthority,
    NetworkIdentity, NetworkReplicationAuthority, NetworkServerAuthority, Role, Transformation,
    Trusted, Untrusted,
};

/// Replication Plugin.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ReplicationPlugin<C, E, P> {
    role: Role,
    _c: PhantomData<C>,
    _e: PhantomData<E>,
    _p: PhantomData<P>,
}

impl<C, E, P> ReplicationPlugin<C, E, P> {
    /// Creates a new [`ReplicationPlugin`].
    #[must_use]
    pub fn new(role: Role) -> Self {
        Self {
            role,
            _c: PhantomData,
            _e: PhantomData,
            _p: PhantomData,
        }
    }
}

impl<C, E, P> Plugin for ReplicationPlugin<C, E, P>
where
    C: ReplicateComponent + Component,
    C::Message: chaos_symphony_protocol::Event<P>,
    E: ReplicateEvent + Clone + Send + Sync + 'static + chaos_symphony_protocol::Event<P>,
    P: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<Trusted<E>>().add_event::<Untrusted<E>>();

        app.add_systems(Update, apply_trusted_event::<E>);

        match self.role {
            Role::Client => {
                app.add_systems(
                    Update,
                    send_untrusted_event::<
                        E,
                        P,
                        EntityReplicationAuthority,
                        NetworkReplicationAuthority,
                    >,
                );
            }
            Role::Replication => {
                app.add_systems(Update, send_trusted_event::<E, P>);
                app.add_systems(
                    Update,
                    send_untrusted_event::<
                        //
                        E,
                        P,
                        EntityServerAuthority,
                        NetworkServerAuthority,
                    >,
                );
                app.add_systems(Update, replicate_trusted_component::<C, P>);
            }
            Role::Simulation => {
                app.add_systems(Update, send_trusted_event::<E, P>);
                app.add_systems(Update, replicate_trusted_component::<C, P>);
            }
        };
    }
}

#[allow(clippy::needless_pass_by_value)]
fn apply_trusted_event<E>(
    mut commands: Commands,
    mut reader: EventReader<Trusted<E>>,
    query: Query<(&EntityIdentity, Entity)>,
) where
    E: ReplicateEvent + Send + Sync + 'static,
{
    reader.read().for_each(|trusted| {
        let span = error_span!("event", message_id =%  trusted.inner.id());
        let _guard = span.enter();

        let Some((_, entity)) = query
            .iter()
            .find(|(entity_identity, _)| entity_identity.inner == *trusted.inner.entity_identity())
        else {
            warn!("entity does not exist");
            return;
        };

        trusted.inner.insert_bundle(commands.entity(entity));
    });
}

#[allow(clippy::needless_pass_by_value)]
fn send_trusted_event<E, P>(
    mut reader: EventReader<Trusted<E>>,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity)>,
) where
    E: ReplicateEvent + Clone + Send + Sync + 'static + chaos_symphony_protocol::Event<P>,
{
    reader.read().for_each(|event| {
        endpoints
            .iter()
            .filter(|(_, network_identity)| {
                network_identity.inner != *event.inner.source_identity()
            })
            .for_each(|(endpoint, _)| {
                let message = event.inner.clone();
                if message.try_send(endpoint).is_err() {
                    error!("failed to send event");
                };
            });
    });
}

#[allow(clippy::needless_pass_by_value)]
fn send_untrusted_event<E, P, EA, NA>(
    mut reader: EventReader<Untrusted<E>>,
    endpoints: Query<(&NetworkEndpoint, &NetworkIdentity), With<NA>>,
    entities: Query<(&EA, &EntityIdentity)>,
) where
    E: ReplicateEvent + Clone + Send + Sync + 'static + chaos_symphony_protocol::Event<P>,
    EA: EntityAuthority + Component,
    NA: Component,
{
    reader.read().for_each(|event| {
        let Some((entity_replication_authority, _)) = entities
            .iter()
            .find(|(_, entity_identity)| entity_identity.inner == *event.inner.entity_identity())
        else {
            error!("entity identity does not exist");
            return;
        };

        let Some((endpoint, _)) = endpoints.iter().find(|(_, network_identity)| {
            network_identity.inner == *entity_replication_authority.identity()
        }) else {
            error!("network identity does not exist");
            return;
        };

        let message = event.inner.clone();
        if message.try_send(endpoint).is_err() {
            error!("failed to send event");
        };
    });
}

#[allow(clippy::needless_pass_by_value)]
fn replicate_trusted_component<C, P>(
    mut reader: EventReader<Trusted<ReplicateEntityComponentsRequest>>,
    endpoints: Query<&NetworkEndpoint>,
    entities: Query<(&C, &EntityIdentity)>,
) where
    C: ReplicateComponent + Component,
    C::Message: chaos_symphony_protocol::Event<P>,
{
    reader.read().for_each(|request| {
        let span = error_span!("replicate_trusted_component", request =? request.inner);
        let _guard = span.enter();

        let Some(source_endpoint_id) = &request.inner.header.source_endpoint_id else {
            error!("request does not have source endpoint id");
            return;
        };

        let Some(endpoint) = endpoints
            .iter()
            .find(|endpoint| endpoint.id() == *source_endpoint_id)
        else {
            error!("network endpoint does not exist");
            return;
        };

        let Some((component, entity_identity)) = entities.iter().find(|(_, entity_identity)| {
            entity_identity.inner == request.inner.payload.entity_identity
        }) else {
            error!("entity identity component does not exist");
            return;
        };

        let message = component.to_message(entity_identity);
        if message.try_send(endpoint).is_err() {
            error!("failed to send event");
        };
    });
}

/// Replicate Component.
pub trait ReplicateComponent {
    /// Message.
    type Message;

    /// To Message.
    fn to_message(&self, entity_identity: &EntityIdentity) -> Self::Message;
}

impl ReplicateComponent for Transformation {
    type Message = TransformationEvent;

    fn to_message(&self, entity_identity: &EntityIdentity) -> Self::Message {
        TransformationEvent::message(
            Uuid::new_v4(),
            TransformationEventPayload {
                entity_identity: entity_identity.inner.clone().into(),
                transformation: (*self).into(),
            },
        )
    }
}

/// Replicate Event.
pub trait ReplicateEvent {
    /// Entity Identity.
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity;

    /// Id.
    fn id(&self) -> Uuid;

    /// Insert Bundle.
    fn insert_bundle(&self, commands: EntityCommands<'_, '_, '_>);

    /// Source Identity.
    fn source_identity(&self) -> &chaos_symphony_protocol::Identity;
}

impl ReplicateEvent for TransformationEvent {
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity {
        &self.payload.entity_identity
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn insert_bundle(&self, mut commands: EntityCommands) {
        let component: Transformation = self.payload.transformation.into();
        commands.insert(component);
    }

    fn source_identity(&self) -> &chaos_symphony_protocol::Identity {
        self.header.source_identity.as_ref().unwrap()
    }
}
