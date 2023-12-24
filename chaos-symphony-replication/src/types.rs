use std::marker::PhantomData;

use bevy::prelude::*;
use chaos_symphony_ecs::{network::NetworkEndpointId, types::Identity};

/// Replicate.
#[derive(Debug, Clone, Component)]
pub struct Replicate<T> {
    /// Identity.
    pub identity: Identity,

    marker: PhantomData<T>,
}

impl<T> Replicate<T> {
    /// Creates a new [`Replicate`].
    pub fn new(identity: Identity) -> Self {
        Self {
            identity,
            marker: PhantomData,
        }
    }
}

/// Network Path.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Component)]
pub struct NetworkPath {
    /// Source.
    pub source: NetworkEndpointId,

    /// Target.
    pub target: NetworkEndpointId,
}
