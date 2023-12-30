use std::marker::PhantomData;

use bevy::{ecs::system::EntityCommands, prelude::*};
use chaos_symphony_protocol::TransformationEvent;

use crate::types::{EntityIdentity, Transformation, Trusted, Untrusted};

/// Replication Plugin.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub struct ReplicationPlugin<C, E> {
    _t: PhantomData<C>,
    _e: PhantomData<E>,
}

impl<C, E> ReplicationPlugin<C, E> {
    /// Creates a new [`ReplicationPlugin`].
    pub fn new() -> Self {
        Self {
            _t: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<C, E> Plugin for ReplicationPlugin<C, E>
where
    C: Component,
    E: Replicate + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<Trusted<E>>().add_event::<Untrusted<E>>();

        app.add_systems(Update, apply_trusted_event::<C, E>);
    }
}

fn apply_trusted_event<C, E>(
    mut commands: Commands,
    mut reader: EventReader<Trusted<E>>,
    query: Query<(&EntityIdentity, Entity)>,
) where
    E: Replicate + Send + Sync + 'static,
{
    reader.read().for_each(|trusted| {
        let Some((_, entity)) = query
            .iter()
            .find(|(entity_identity, _)| entity_identity.inner == *trusted.inner.entity_identity())
        else {
            return;
        };

        trusted.inner.insert_bundle(commands.entity(entity));
    });
}

/// Replicate.
pub trait Replicate {
    /// Entity Identity.
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity;

    /// Insert Bundle.
    fn insert_bundle(&self, commands: EntityCommands<'_, '_, '_>);
}

impl Replicate for TransformationEvent {
    fn entity_identity(&self) -> &chaos_symphony_protocol::Identity {
        &self.payload.entity_identity
    }

    fn insert_bundle(&self, mut commands: EntityCommands<'_, '_, '_>) {
        let component: Transformation = self.payload.transformation.clone().into();
        commands.insert(component);
    }
}
