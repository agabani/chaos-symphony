use std::marker::PhantomData;

use bevy::{ecs::system::EntityCommands, prelude::*, utils::Uuid};
use chaos_symphony_network_bevy::NetworkEndpoint;
use chaos_symphony_protocol::TransformationEvent;

use crate::types::{
    EntityAuthority, EntityIdentity, EntityReplicationAuthority, EntityServerAuthority,
    NetworkIdentity, NetworkReplicationAuthority, NetworkServerAuthority, Transformation, Trusted,
    Untrusted,
};

/// Replication Plugin.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ReplicationPlugin<E, P> {
    mode: ReplicationMode,
    _e: PhantomData<E>,
    _p: PhantomData<P>,
}

impl<E, P> ReplicationPlugin<E, P> {
    /// Creates a new [`ReplicationPlugin`].
    #[must_use]
    pub fn new(mode: ReplicationMode) -> Self {
        Self {
            mode,
            _e: PhantomData,
            _p: PhantomData,
        }
    }
}

impl<E, P> Plugin for ReplicationPlugin<E, P>
where
    E: Replicate + Clone + Send + Sync + 'static + chaos_symphony_protocol::Event<P>,
    P: Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<Trusted<E>>().add_event::<Untrusted<E>>();

        app.add_systems(Update, apply_trusted_event::<E>);

        match self.mode {
            ReplicationMode::Client => {
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
            ReplicationMode::Replication => {
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
            }
            ReplicationMode::Simulation => {
                app.add_systems(Update, send_trusted_event::<E, P>);
            }
        };
    }
}

/// Replication Mode.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy)]
pub enum ReplicationMode {
    /// Client.
    Client,

    /// Replication.
    Replication,

    /// Simulation.
    Simulation,
}

#[allow(clippy::needless_pass_by_value)]
fn apply_trusted_event<E>(
    mut commands: Commands,
    mut reader: EventReader<Trusted<E>>,
    query: Query<(&EntityIdentity, Entity)>,
) where
    E: Replicate + Send + Sync + 'static,
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
    E: Replicate + Clone + Send + Sync + 'static + chaos_symphony_protocol::Event<P>,
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
    E: Replicate + Clone + Send + Sync + 'static + chaos_symphony_protocol::Event<P>,
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
            network_identity.inner != *entity_replication_authority.identity()
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

/// Replicate.
pub trait Replicate {
    /// Entity Identity.
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity;

    /// Id.
    fn id(&self) -> Uuid;

    /// Insert Bundle.
    fn insert_bundle(&self, commands: EntityCommands<'_, '_, '_>);

    /// Source Identity.
    fn source_identity(&self) -> &chaos_symphony_protocol::Identity;
}

impl Replicate for TransformationEvent {
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
