#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony ECS

/// Entity Identities.
pub mod entity_identities;
/// Network.
pub mod network;
/// Network Authenticate.
pub mod network_authenticate;
/// Network Authority.
pub mod network_authority;
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
    pub identity: types::NetworkIdentity,
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
            network_authority::NetworkAuthorityPlugin,
            network_connect::NetworkConnectPlugin,
            network_disconnect::NetworkDisconnectPlugin,
            network_keep_alive::NetworkKeepAlivePlugin,
        ));

        app.add_plugins(entity_identities::EntityIdentitiesPlugin);

        app.register_type::<bevy::utils::Uuid>()
            .register_type::<types::Identity>()
            .register_type::<types::EntityIdentity>()
            .register_type::<types::EntityClientAuthority>()
            .register_type::<types::EntityServerAuthority>()
            .register_type::<types::NetworkIdentity>()
            .register_type::<types::NetworkClientAuthority>()
            .register_type::<types::NetworkServerAuthority>()
            .register_type::<types::Transformation>();
    }
}
