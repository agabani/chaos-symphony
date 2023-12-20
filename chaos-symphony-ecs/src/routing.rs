use bevy::prelude::*;

/// Endpoint Id.
#[derive(Component)]
pub struct EndpointId {
    /// Inner.
    pub inner: usize,
}

/// Request.
#[derive(Component)]
pub struct Request<T> {
    /// Inner.
    pub inner: T,
}
