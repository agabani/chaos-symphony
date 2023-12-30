#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony ECS

/// Bevy Config.
pub mod bevy_config;
/// Entity Identities.
pub mod entity_identities;
/// Entity Identity.
pub mod entity_identity;
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
/// Replicate Entity Components.
pub mod replicate_entity_components;
/// Replication.
pub mod replication;
/// Transformation.
pub mod transformation;
/// Types.
pub mod types;

/// Default Plugins.
pub struct DefaultPlugins {
    /// Bevy Config.
    pub bevy_config: bevy_config::BevyConfigPlugin,

    /// Network Authenticate.
    pub network_authenticate: network_authenticate::NetworkAuthenticatePlugin,
}

impl bevy::prelude::Plugin for DefaultPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(self.bevy_config.clone());

        app.add_plugins((
            chaos_symphony_network_bevy::NetworkPlugin {
                client: true,
                server: false,
            },
            self.network_authenticate.clone(),
            network_authority::NetworkAuthorityPlugin,
            network_connect::NetworkConnectPlugin,
            network_disconnect::NetworkDisconnectPlugin,
            network_keep_alive::NetworkKeepAlivePlugin,
        ));

        app.add_plugins((
            entity_identities::EntityIdentitiesPlugin,
            entity_identity::EntityIdentityPlugin,
            replicate_entity_components::ReplicateEntityComponentsPlugin,
            transformation::TransformationPlugin,
        ));

        app.register_type::<bevy::utils::Uuid>()
            .register_type::<types::Identity>()
            .register_type::<types::EntityIdentity>()
            .register_type::<types::EntityClientAuthority>()
            .register_type::<types::EntityServerAuthority>()
            .register_type::<types::NetworkIdentity>()
            .register_type::<types::NetworkClientAuthority>()
            .register_type::<types::NetworkServerAuthority>()
            .register_type::<types::Transformation>();

        // SPIKE IN PROGRESS
        app.add_plugins(replication::ReplicationPlugin::<
            types::Transformation,
            chaos_symphony_protocol::TransformationEvent,
        >::new());
    }
}
