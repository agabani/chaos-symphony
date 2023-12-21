use bevy::{
    math::{DQuat, DVec3},
    prelude::*,
};

/// Transformation.
#[derive(Component, Reflect)]
pub struct Transformation {
    /// Orientation.
    pub orientation: DQuat,

    /// Position.
    pub position: DVec3,
}
