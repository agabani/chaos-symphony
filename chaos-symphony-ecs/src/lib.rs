#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony ECS

/// Network.
pub mod network;
/// Network Authenticate.
pub mod network_authenticate;
/// Network Connect.
pub mod network_connect;
/// Network Disconnect.
pub mod network_disconnect;
/// Network Keep Alive.
pub mod network_keep_alive;
/// Types.
pub mod types;

/// Default Plugins.
pub struct DefaultPlugins {
    /// Identity.
    pub identity: types::Identity,
}

impl bevy::prelude::Plugin for DefaultPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            chaos_symphony_network_bevy::NetworkPlugin {
                client: true,
                server: false,
            },
            network_authenticate::NetworkAuthenticatePlugin {
                identity: self.identity.clone(),
            },
            network_connect::NetworkConnectPlugin,
            network_disconnect::NetworkDisconnectPlugin,
            network_keep_alive::NetworkKeepAlivePlugin,
        ));

        app.register_type::<types::ClientAuthority>()
            .register_type::<types::ServerAuthority>()
            .register_type::<types::Identity>()
            .register_type::<types::Transformation>()
            .register_type::<bevy::utils::Uuid>();
    }
}
