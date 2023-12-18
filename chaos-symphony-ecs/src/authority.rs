use bevy::prelude::*;

/// Client Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Component)]
pub struct ClientAuthority {
    inner: String,
}

impl ClientAuthority {
    /// Creates a new [`ClientAuthority`].
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

/// Server Authority.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Component)]
pub struct ServerAuthority {
    inner: String,
}

impl ServerAuthority {
    /// Creates a new [`ServerAuthority`].
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
