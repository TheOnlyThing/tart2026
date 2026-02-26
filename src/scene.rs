use bevy::{camera_controller::free_camera::FreeCamera, ecs::name, platform::collections::{HashMap, HashSet}, prelude::*};
use crate::code::*;
//time
use std::time::Instant;

//byte event shi
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write, Read, Seek, SeekFrom};
use std::path::Path;

const EVENT_SIZE: usize = 36; // 16 (u128) + 8 (u64) + 12 (Vec3<f32>) bytes

//observer spawn test
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;


//plugin stuffs ==

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_scene);
        app.add_systems(FixedUpdate, (draw_shapes));
        app.add_systems(FixedUpdate, (faux_movement));

        app.init_resource::<NearbyIndex>();
        app.add_observer(
        |explode_mines: On<ExplodeMines>,
        mines: Query<(Entity, &Mine)>,
        mut commands: Commands| {
        for (entity, mine) in &mines {
            if mine.pos.distance(explode_mines.pos) < mine.size + explode_mines.radius {
                commands.trigger(Explode { entity });
            }
        }
    },
);

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

#[derive(Resource, Default)]
struct NearbyIndex {
    map: HashMap<(i32, i32), HashSet<Entity>>,
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
    for _ in 0..1000 {
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

// Draw a circle for each mine using `Gizmos`
fn draw_shapes(mut gizmos: Gizmos, mines: Query<&Mine>) {
    for mine in &mines {
        gizmos.sphere(
            mine.pos,
            mine.size,
            Color::hsl((mine.size - 4.0) / 16.0 * 360.0, 1.0, 0.8),
        );
    }
}

fn faux_movement(
    query: Query<(Entity, &Transform), (With<Camera>, Changed<Transform>)>,
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut last_sample: Local<Option<(Vec3, f64)>>,
) {
    let radius = 20.0;

    for (entity, transform) in query.iter() {
        let center = transform.translation;

        // current time in nano seconds
        let start = Instant::now();
        let elapsed_nanos: u128 = start.elapsed().as_millis();
        
        //append vec3
        if let Ok(mut log) = EventLog::open("position_events.log") {
            let event: PositionEvent = PositionEvent {
                tick: elapsed_nanos,
                entity_id: entity.to_bits(),
                position: center,
            };
            let _ = log.append(event);
            println!("{:?}", event);
        }

        //println!("force:{:?}",force);
        //gizmos.line(center, center + force, Color::hsl(200.0, 1.0, 0.6));
        
        gizmos.sphere(
            center,
            radius,
            Color::hsl(21.0,1.0,0.6),
        );

        commands.trigger(ExplodeMines { pos: center, radius: radius });
    }
}

//understand
//converter from position data to bytes shi
#[derive(Debug, Clone, Copy)]
pub struct PositionEvent {
    pub tick: u128,
    pub entity_id: u64,
    pub position: Vec3,
}

impl PositionEvent {
    pub fn to_bytes(self) -> [u8; EVENT_SIZE] {
        let mut buf = [0u8; EVENT_SIZE];

        buf[0..16].copy_from_slice(&self.tick.to_le_bytes());
        buf[16..24].copy_from_slice(&self.entity_id.to_le_bytes());
        buf[24..28].copy_from_slice(&self.position.x.to_le_bytes());
        buf[28..32].copy_from_slice(&self.position.y.to_le_bytes());
        buf[32..36].copy_from_slice(&self.position.z.to_le_bytes());

        buf
    }

    pub fn from_bytes(buf: &[u8; EVENT_SIZE]) -> Self {
        let tick = u128::from_le_bytes(buf[0..16].try_into().unwrap());
        let entity_id = u64::from_le_bytes(buf[16..24].try_into().unwrap());
        let x = f32::from_le_bytes(buf[24..28].try_into().unwrap());
        let y = f32::from_le_bytes(buf[28..32].try_into().unwrap());
        let z = f32::from_le_bytes(buf[32..36].try_into().unwrap());

        Self {
            tick,
            entity_id,
            position: Vec3::new(x, y, z),
        }
    }
}

//understand
//file shi and maybe reading for the byte file idk
pub struct EventLog {
    writer: BufWriter<File>,
}

impl EventLog {
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            writer: BufWriter::with_capacity(1024 * 1024, file), // 1MB buffer
        })
    }

    pub fn append(&mut self, event: PositionEvent) -> std::io::Result<()> {
        let bytes = event.to_bytes();
        self.writer.write_all(&bytes)
    }

    pub fn replay(path: impl AsRef<Path>) -> std::io::Result<Vec<PositionEvent>> {
        let mut file = File::open(path)?;   // ← Opens the file for reading
        let mut events = Vec::new();
        let mut buf = [0u8; EVENT_SIZE];

        loop {
            match file.read_exact(&mut buf) {  // ← Reads raw bytes
                Ok(_) => {
                    events.push(PositionEvent::from_bytes(&buf)); // ← Converts bytes → struct
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }

        Ok(events)
    }
}
