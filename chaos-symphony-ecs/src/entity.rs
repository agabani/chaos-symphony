use bevy::prelude::*;

/// Identifier.
#[derive(Component)]
pub struct Identifier {
    inner: String,
}

impl Identifier {
    /// Creates a new [`Identifier`].
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
