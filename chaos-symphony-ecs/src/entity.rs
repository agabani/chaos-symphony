use bevy::prelude::*;

/// Identity.
#[derive(Debug, Component)]
pub struct Identity {
    inner: String,
}

impl Identity {
    /// Creates a new [`Identity`].
    #[must_use]
    pub fn new(id: String) -> Self {
        Self { inner: id }
    }

    /// Id.
    #[must_use]
    pub fn id(&self) -> &str {
        self.inner.as_ref()
    }
}
