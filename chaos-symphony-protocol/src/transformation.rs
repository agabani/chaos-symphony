use bevy::math::{DQuat, DVec3};
use serde::{Deserialize, Serialize};

/// Orientation.
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Orientation {
    /// X.
    pub x: f64,

    /// Y.
    pub y: f64,

    /// Z.
    pub z: f64,

    /// W.
    pub w: f64,
}

impl Default for Orientation {
    fn default() -> Self {
        DQuat::default().into()
    }
}

impl From<DQuat> for Orientation {
    fn from(value: DQuat) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<Orientation> for DQuat {
    fn from(value: Orientation) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

/// Position.
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Position {
    /// X.
    pub x: f64,

    /// Y.
    pub y: f64,

    /// Z.
    pub z: f64,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl From<DVec3> for Position {
    fn from(value: DVec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Position> for DVec3 {
    fn from(value: Position) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

/// Position.
#[derive(Default, Clone, Copy, Deserialize, Serialize)]
pub struct Transformation {
    /// Orientation.
    pub orientation: Orientation,

    /// Position.
    pub position: Position,
}
