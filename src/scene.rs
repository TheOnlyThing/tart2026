use bevy::{camera_controller::free_camera::FreeCamera, ecs::name, prelude::*};
use crate::code::*;

//observer spawn test
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;


//plugin stuffs ==

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
    }
}

//components ===========

#[derive(EntityEvent)]
struct Explode {
    entity: Entity,
}

#[derive(Event)]
struct ExplodeMines {
    pos: Vec3,
    radius: f32,
}



#[derive(Component)]
struct Mine {
    pos: Vec3,
    size: f32,
}

impl Mine {
    fn random(rand: &mut ChaCha8Rng) -> Self {
        Mine {
            pos: Vec3::new(
                (rand.random::<f32>() - 0.5) * 1200.0,
                (rand.random::<f32>() - 0.5) * 600.0,
                (rand.random::<f32>() - 0.5) * 600.0,
            ),
            size: 4.0 + rand.random::<f32>() * 16.0,
        }
    }
}



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
        FreeCamera{..Default::default()},
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


    let mut rng = ChaCha8Rng::seed_from_u64(19878367467713);

    //observer
    let mut observer = Observer::new(explode_mine);

    // As we spawn entities, we can make this observer watch each of them:
    for _ in 0..100 {
        let entity = commands.spawn(
            Mine::random(&mut rng)
        ).id();
        commands.entity(entity).insert(Transform::from_xyz( 0.5, 0.0,0.0));
        observer.watch_entity(entity);
    }

    // By spawning the Observer component, it becomes active!
    commands.spawn(observer);

}

fn explode_mine(explode: On<Explode>, query: Query<&Mine>, mut commands: Commands) {
    // Explode is an EntityEvent. `explode.entity` is the entity that Explode was triggered for.
    let Ok(mut entity) = commands.get_entity(explode.entity) else {
        return;
    };
    info!("Boom! {} exploded.", explode.entity);
    entity.despawn();
    let mine = query.get(explode.entity).unwrap();
    // Trigger another explosion cascade.
    commands.trigger(ExplodeMines {
        pos: mine.pos,
        radius: mine.size,
    });
}
