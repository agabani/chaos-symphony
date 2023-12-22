use std::fmt::Display;

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
