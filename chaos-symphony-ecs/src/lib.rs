#![deny(clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

//! Chaos Symphony ECS

/// Bevy Config.
pub mod bevy_config;
/// Entity Identities.
pub mod entity_identities;
/// Entity Identity.
pub mod entity_identity;
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
/// Network Router.
pub mod network_router;
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

    /// Network Identity.
    pub network_identity: types::NetworkIdentity,

    /// Role.
    pub role: types::Role,
}

impl bevy::prelude::Plugin for DefaultPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(self.bevy_config.clone());

        // network
        app.add_plugins((
            chaos_symphony_network_bevy::NetworkPlugin {
                client: match self.role {
                    types::Role::Client | types::Role::Simulation => true,
                    types::Role::Replication => false,
                },
                server: match self.role {
                    types::Role::Client | types::Role::Simulation => false,
                    types::Role::Replication => true,
                },
            },
            network_authenticate::NetworkAuthenticatePlugin::new(
                self.network_identity.clone(),
                self.role,
            ),
            network_authority::NetworkAuthorityPlugin,
            network_connect::NetworkConnectPlugin::new(self.role),
            network_disconnect::NetworkDisconnectPlugin,
            network_keep_alive::NetworkKeepAlivePlugin::new(self.role),
            network_router::NetworkRouter,
        ));

        // entity
        app.add_plugins((
            entity_identities::EntityIdentitiesPlugin::new(self.role),
            entity_identity::EntityIdentityPlugin::new(self.role),
        ));

        // components
        app.add_plugins(transformation::TransformationPlugin::new(self.role));

        // replication
        app.add_plugins(
            replicate_entity_components::ReplicateEntityComponentsPlugin::new(self.role),
        );

        app.add_plugins(replication::ReplicationPlugin::<
            types::EntityClientAuthority,
            chaos_symphony_protocol::EntityClientAuthorityEvent,
            chaos_symphony_protocol::EntityClientAuthorityEventPayload,
        >::new(self.role));

        app.add_plugins(replication::ReplicationPlugin::<
            types::EntityReplicationAuthority,
            chaos_symphony_protocol::EntityReplicationAuthorityEvent,
            chaos_symphony_protocol::EntityReplicationAuthorityEventPayload,
        >::new(self.role));

        app.add_plugins(replication::ReplicationPlugin::<
            types::EntitySimulationAuthority,
            chaos_symphony_protocol::EntitySimulationAuthorityEvent,
            chaos_symphony_protocol::EntitySimulationAuthorityEventPayload,
        >::new(self.role));

        app.add_plugins(replication::ReplicationPlugin::<
            types::Transformation,
            chaos_symphony_protocol::TransformationEvent,
            chaos_symphony_protocol::TransformationEventPayload,
        >::new(self.role));

        // type
        app.register_type::<bevy::utils::Uuid>()
            .register_type::<types::Identity>()
            .register_type::<types::EntityIdentity>()
            .register_type::<types::EntityClientAuthority>()
            .register_type::<types::EntityReplicationAuthority>()
            .register_type::<types::EntitySimulationAuthority>()
            .register_type::<types::NetworkIdentity>()
            .register_type::<types::NetworkClientAuthority>()
            .register_type::<types::NetworkServerAuthority>()
            .register_type::<types::Transformation>();
    }
}
