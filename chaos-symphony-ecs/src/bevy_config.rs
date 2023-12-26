use bevy::{log::LogPlugin, prelude::*};
use tracing::Level;

/// Bevy Config Plugin.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct BevyConfigPlugin {
    /// Headless.
    pub headless: bool,

    /// Log Filter.
    pub log_filter: String,

    /// Title.
    pub title: String,
}

impl Plugin for BevyConfigPlugin {
    fn build(&self, app: &mut App) {
        let log_filter = format!("{}=debug", self.log_filter);

        if self.headless {
            app.add_plugins((
                MinimalPlugins,
                LogPlugin {
                    filter: [
                        "info",
                        "chaos_symphony_ecs=debug",
                        "chaos_symphony_network_bevy=debug",
                        &log_filter,
                        "wgpu_core=warn",
                        "wgpu_hal=warn",
                    ]
                    .join(","),
                    level: Level::DEBUG,
                },
            ));
        } else {
            app.add_plugins(
                DefaultPlugins
                    .set(LogPlugin {
                        filter: [
                            "info",
                            "chaos_symphony_ecs=debug",
                            "chaos_symphony_network_bevy=debug",
                            "chaos_symphony=debug",
                            "wgpu_core=warn",
                            "wgpu_hal=warn",
                        ]
                        .join(","),
                        level: Level::DEBUG,
                    })
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: self.title.clone(),
                            ..default()
                        }),
                        ..default()
                    }),
            )
            .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
        }
    }
}
