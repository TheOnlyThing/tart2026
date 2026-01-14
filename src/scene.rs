use bevy::{ecs::name, prelude::*};
use crate::code::*;

//plugin stuffs ==

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
    }
}

//components ===========


//systems ============

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(7.5, 0.0, 25.0),
    ));
    //blue_thing
    commands.spawn((
        Name::new("joe"),
        d1_position { pos: 0.5 },
        d1_size { size: 3.0 },

        Mesh3d(meshes.add(Cuboid::new(3.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz( 0.5, 0.0,0.0),
    ));

}