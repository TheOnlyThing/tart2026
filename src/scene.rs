use bevy::{camera_controller::free_camera::FreeCamera, ecs::name, input::{ButtonState, keyboard::KeyboardInput}, platform::collections::{HashMap, HashSet}, prelude::*};
use crate::code::*;
//time
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

//byte event shi
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write, Read, Seek, SeekFrom};
use std::path::Path;

const EVENT_SIZE: usize = 36; // 16 (u128) + 8 (u64) + 12 (Vec3<f32>) bytes
const INPUT_EVENT_SIZE: usize = 25; // 16 (u128) + 8 (u64 key hash) + 1 (bool) bytes

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
        app.add_systems(FixedUpdate, (input_log));

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
    for _ in 0..10 {
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

fn input_log(mut keyboard_inputs: MessageReader<KeyboardInput>) {
    for input in keyboard_inputs.read() {

        // current time in nano seconds
        let start = Instant::now();
        let elapsed_nanos: u128 = start.elapsed().as_nanos();

        info!("time: {:?} keycode: {:?} state: {:?}", elapsed_nanos,input.key_code,input.state,);
        //append input
        if let Ok(mut log) = InputLog::open("input_events.log") {
            let event: InputEvent = InputEvent {
                nanos: elapsed_nanos,
                key_code_hash: hash_key_code(&input.key_code),
                input_state: input.state.is_pressed(),
            };
            let _ = log.append(event);
            input_reader();
            //println!("{:?}", event);
        }
    }
}

//maaybe dont register input from reading, raather just store? but I dont want two systems so idk
fn input_reader() {
    match InputLog::replay("input_events.log") {
        Ok(events) => {
            if events.is_empty() {
                println!("input_events.log is empty");
                return;
            }

            println!("Read {} input events from input_events.log", events.len());
            for (i, event) in events.iter().enumerate() {
                let state = if event.input_state { "Pressed" } else { "Released" };
                println!(
                    "[{i}] nanos={} key_hash={} state={}",
                    event.nanos, event.key_code_hash, state
                );
            }
        }
        Err(err) => eprintln!("Failed to read input_events.log: {err}"),
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
        let elapsed_nanos: u128 = start.elapsed().as_nanos();
        
        //append vec3
        if let Ok(mut log) = EventLog::open("position_events.log") {
            let event: PositionEvent = PositionEvent {
                tick: elapsed_nanos,
                entity_id: entity.to_bits(),
                position: center,
            };
            let _ = log.append(event);
            //println!("{:?}", event);
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
#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub nanos: u128,
    pub key_code_hash: u64,
    pub input_state: bool,
}

//why does bool need 24 bits?
impl InputEvent {
    pub fn to_bytes(self) -> [u8; INPUT_EVENT_SIZE] {
        let mut buf = [0u8; INPUT_EVENT_SIZE];
        buf[0..16].copy_from_slice(&self.nanos.to_le_bytes());
        buf[16..24].copy_from_slice(&self.key_code_hash.to_le_bytes());
        buf[24] = self.input_state as u8;
        buf
    }

    pub fn from_bytes(buf: &[u8; INPUT_EVENT_SIZE]) -> Self {
        let nanos = u128::from_le_bytes(buf[0..16].try_into().unwrap());
        let key_code_hash = u64::from_le_bytes(buf[16..24].try_into().unwrap());
        let input_state = buf[24] != 0;

        Self {
            nanos,
            key_code_hash,
            input_state,
        }
    }
}

//is this best way to convert KeyCode data? when would we want to convert types an in what way
fn hash_key_code(key_code: &KeyCode) -> u64 {
    let mut hasher = DefaultHasher::new();
    key_code.hash(&mut hasher);
    hasher.finish()
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

pub struct InputLog {
    writer: BufWriter<File>,
}

fn open_log_writer(path: impl AsRef<Path>) -> std::io::Result<BufWriter<File>> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(BufWriter::with_capacity(1024 * 1024, file))
}

fn replay_fixed_size<T, const N: usize>(
    path: impl AsRef<Path>,
    from_bytes: fn(&[u8; N]) -> T,
) -> std::io::Result<Vec<T>> {
    let mut file = File::open(path)?;
    let mut events = Vec::new();
    let mut buf = [0u8; N];

    loop {
        match file.read_exact(&mut buf) {
            Ok(_) => events.push(from_bytes(&buf)),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }

    Ok(events)
}

impl EventLog {
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self {
            writer: open_log_writer(path)?,
        })
    }

    pub fn append(&mut self, event: PositionEvent) -> std::io::Result<()> {
        self.writer.write_all(&event.to_bytes())
    }

    pub fn replay(path: impl AsRef<Path>) -> std::io::Result<Vec<PositionEvent>> {
        replay_fixed_size::<PositionEvent, EVENT_SIZE>(path, PositionEvent::from_bytes)
    }
}

impl InputLog {
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self {
            writer: open_log_writer(path)?,
        })
    }

    pub fn append(&mut self, event: InputEvent) -> std::io::Result<()> {
        self.writer.write_all(&event.to_bytes())
    }

    pub fn replay(path: impl AsRef<Path>) -> std::io::Result<Vec<InputEvent>> {
        replay_fixed_size::<InputEvent, INPUT_EVENT_SIZE>(path, InputEvent::from_bytes)
    }
}
