use std::marker::PhantomData;

use bevy::{ecs::system::EntityCommands, prelude::*, utils::Uuid};
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
    #[must_use]
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

#[allow(clippy::needless_pass_by_value)]
fn apply_trusted_event<C, E>(
    mut commands: Commands,
    mut reader: EventReader<Trusted<E>>,
    query: Query<(&EntityIdentity, Entity)>,
) where
    E: Replicate + Send + Sync + 'static,
{
    reader.read().for_each(|trusted| {
        let span = error_span!("event", message_id =%  trusted.inner.id());
        let _guard = span.enter();

        let Some((entity_identity, entity)) = query
            .iter()
            .find(|(entity_identity, _)| entity_identity.inner == *trusted.inner.entity_identity())
        else {
            warn!("entity does not exist");
            return;
        };

        trusted.inner.insert_bundle(commands.entity(entity));
        info!(entity_identity =? entity_identity, "updated");
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
}
