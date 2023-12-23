use std::fmt::Display;

use bevy::math::{DQuat, DVec3};
use bevy::utils::Uuid;
use serde::{Deserialize, Serialize};

/// Identity.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Identity {
    /// Id.
    pub id: Uuid,

    /// Noun.
    pub noun: String,
}

impl Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.noun, self.id)
    }
}

/// Orientation.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x:{}, y:{}, z:{}, w:{}", self.x, self.y, self.z, self.w)
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x:{}, y:{}, z:{}", self.x, self.y, self.z)
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
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct Transformation {
    /// Orientation.
    pub orientation: Orientation,

    /// Position.
    pub position: Position,
}

impl Display for Transformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "orientation:[{}], position:[{}]",
            self.orientation, self.position
        )
    }
}
