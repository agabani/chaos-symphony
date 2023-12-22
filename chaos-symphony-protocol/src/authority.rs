use bevy::utils::Uuid;
use serde::{Deserialize, Serialize};

/// Identity.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Deserialize, Serialize)]
pub struct Identity {
    /// Id.
    pub id: Uuid,

    /// Noun.
    pub noun: String,
}

impl Identity {
    /// Creates a new zeroed [`Identity`].
    #[must_use]
    pub fn zero() -> Self {
        Self {
            id: Uuid::nil(),
            noun: String::new(),
        }
    }
}
