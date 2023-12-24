use bevy::{prelude::*, render::mesh::shape::RegularPolygon, sprite::MaterialMesh2dBundle};
use chaos_symphony_ecs::{
    ship::Ship,
    types::{EntityClientAuthority, EntityServerAuthority},
};

/// Ship Plugin.
#[allow(clippy::module_name_repetitions)]
pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, added);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn added(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ships: Query<(Entity, &EntityClientAuthority, &EntityServerAuthority), Added<Ship>>,
) {
    ships.for_each(|(entity, client_authority, server_authority)| {
        commands
            .entity(entity)
            .insert(InheritedVisibility::VISIBLE)
            .with_children(|parent| {
                // model
                parent.spawn(MaterialMesh2dBundle {
                    mesh: meshes.add(RegularPolygon::new(8.0, 3).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                    ..default()
                });

                // client authority
                let rgb = client_authority.identity().id().as_u128() % (256 * 256 * 256);
                let b = rgb & 0xff;
                let g = (rgb >> 8) & 0xff;
                let r = (rgb >> 16) & 0xff;
                let color = Color::rgb_u8(r as u8, g as u8, b as u8);

                parent.spawn(MaterialMesh2dBundle {
                    mesh: meshes.add(RegularPolygon::new(16.0, 3).into()).into(),
                    material: materials.add(ColorMaterial::from(color)),
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, -0.1),
                        ..default()
                    },
                    ..default()
                });

                // server authority
                let rgb = server_authority.identity().id().as_u128() % (256 * 256 * 256);
                let b = rgb & 0xff;
                let g = (rgb >> 8) & 0xff;
                let r = (rgb >> 16) & 0xff;
                let color = Color::rgb_u8(r as u8, g as u8, b as u8);

                parent.spawn(MaterialMesh2dBundle {
                    mesh: meshes.add(RegularPolygon::new(24.0, 3).into()).into(),
                    material: materials.add(ColorMaterial::from(color)),
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, -0.2),
                        ..default()
                    },
                    ..default()
                });
            });
    });
}
