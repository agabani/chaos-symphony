use bevy::prelude::*;
use chaos_symphony_ecs::transform::Transformation;

/// Transformation Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct TransformationPlugin;

impl Plugin for TransformationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (added, changed));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn added(mut commands: Commands, query: Query<(Entity, &Transformation), Added<Transformation>>) {
    query.for_each(|(entity, transform)| {
        commands.entity(entity).insert(TransformBundle {
            local: Transform {
                translation: transform.position.as_vec3(),
                rotation: transform.orientation.as_f32(),
                ..Default::default()
            },
            ..Default::default()
        });
    });
}

fn changed(mut query: Query<(&Transformation, &mut Transform), Changed<Transformation>>) {
    query.for_each_mut(|(transformation, mut transform)| {
        transform.translation = transformation.position.as_vec3();
        transform.rotation = transformation.orientation.as_f32();
    });
}
