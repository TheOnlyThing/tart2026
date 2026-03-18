use bevy::{camera_controller::free_camera::FreeCamera, ecs::name, input::{ButtonState, keyboard::KeyboardInput}, platform::collections::{HashMap, HashSet}, prelude::*};
use crate::code::*;
//time
use std::time::Instant;

//byte event shi
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write, Read, Seek, SeekFrom};
use std::path::Path;

const EVENT_SIZE: usize = 36; // 16 (u128) + 8 (u64) + 12 (Vec3<f32>) bytes
const INPUT_EVENT_SIZE: usize = 21; // 16 (u128) + 4 (u32 key id) + 1 (bool) bytes

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
        |explode_specks: On<ExplodeSpecks>,
        specks: Query<(Entity, &Speck)>,
        mut commands: Commands| {
        for (entity, speck) in &specks {
            if speck.pos.distance(explode_specks.pos) < speck.size + explode_specks.radius {
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
struct ExplodeSpecks {
    pos: Vec3,
    radius: f32,
}


#[derive(Component)]
struct Speck {
    pos: Vec3,
    size: f32,
}

#[derive(Resource)]
struct StartTime(Instant);

impl Speck {
    fn random(rand: &mut ChaCha8Rng) -> Self {
        Speck {
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

    commands.insert_resource(StartTime(Instant::now()));

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
    let mut observer = Observer::new(explode_speck);

    // As we spawn entities, we can make this observer watch each of them:
    for _ in 0..10 {
        let entity = commands.spawn(
            Speck::random(&mut rng)
        ).id();
        commands.entity(entity).insert(Transform::from_xyz( 0.5, 0.0,0.0));
        observer.watch_entity(entity);
    }

    // By spawning the Observer component, it becomes active!
    commands.spawn(observer);

}




fn explode_speck(explode: On<Explode>, specks: Query<&Speck>, mut commands: Commands) {
    // Explode is an EntityEvent. `explode.entity` is the entity that Explode was triggered for.
    let Ok(speck) = specks.get(explode.entity) else { return; };

    info!("Boom! {} exploded.", explode.entity);
    commands.entity(explode.entity).despawn();

    // Trigger another explosion cascade.
    commands.trigger(ExplodeSpecks {
        pos: speck.pos,
        radius: speck.size,
    });

}

// Draw a circle for each speck using `Gizmos`
fn draw_shapes(mut gizmos: Gizmos, specks: Query<&Speck>) {
    for speck in &specks {
        gizmos.sphere(
            speck.pos,
            speck.size,
            Color::hsl((speck.size - 4.0) / 16.0 * 360.0, 1.0, 0.8),
        );
    }
}

fn input_log(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    start_time: Res<StartTime>,
) {
    for input in keyboard_inputs.read() {

        // current time in nano seconds
        let elapsed_nanos: u128 = start_time.0.elapsed().as_nanos();

        info!("time: {:?} keycode: {:?} state: {:?}", elapsed_nanos,input.key_code,input.state,);
        //append input
        if let Ok(mut log) = InputLog::open("input_events.log") {
            let event: InputEvent = InputEvent {
                nanos: elapsed_nanos,
                key_code_id: key_code_to_u32(&input.key_code),
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
                    event.nanos, event.key_code_id, state
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
    start_time: Res<StartTime>,
) {
    let radius = 20.0;

    for (entity, transform) in query.iter() {
        let center = transform.translation;

        // current time in nano seconds
        let elapsed_nanos: u128 = start_time.0.elapsed().as_nanos();
        
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

        commands.trigger(ExplodeSpecks { pos: center, radius: radius });
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
    pub key_code_id: u32,
    pub input_state: bool,
}

//why does bool need 24 bits?
impl InputEvent {
    pub fn to_bytes(self) -> [u8; INPUT_EVENT_SIZE] {
        let mut buf = [0u8; INPUT_EVENT_SIZE];
        buf[0..16].copy_from_slice(&self.nanos.to_le_bytes());
        buf[16..20].copy_from_slice(&self.key_code_id.to_le_bytes());
        buf[20] = self.input_state as u8;
        buf
    }

    pub fn from_bytes(buf: &[u8; INPUT_EVENT_SIZE]) -> Self {
        let nanos = u128::from_le_bytes(buf[0..16].try_into().unwrap());
        let key_code_id = u32::from_le_bytes(buf[16..20].try_into().unwrap());
        let input_state = buf[20] != 0;

        Self {
            nanos,
            key_code_id,
            input_state,
        }
    }
}

// Stable mapping from KeyCode to a fixed u32 id. Values must never change once assigned.
fn key_code_to_u32(key_code: &KeyCode) -> u32 {
    match key_code {
        KeyCode::Backquote => 1,
        KeyCode::Backslash => 2,
        KeyCode::BracketLeft => 3,
        KeyCode::BracketRight => 4,
        KeyCode::Comma => 5,
        KeyCode::Digit0 => 6,
        KeyCode::Digit1 => 7,
        KeyCode::Digit2 => 8,
        KeyCode::Digit3 => 9,
        KeyCode::Digit4 => 10,
        KeyCode::Digit5 => 11,
        KeyCode::Digit6 => 12,
        KeyCode::Digit7 => 13,
        KeyCode::Digit8 => 14,
        KeyCode::Digit9 => 15,
        KeyCode::Equal => 16,
        KeyCode::IntlBackslash => 17,
        KeyCode::IntlRo => 18,
        KeyCode::IntlYen => 19,
        KeyCode::KeyA => 20,
        KeyCode::KeyB => 21,
        KeyCode::KeyC => 22,
        KeyCode::KeyD => 23,
        KeyCode::KeyE => 24,
        KeyCode::KeyF => 25,
        KeyCode::KeyG => 26,
        KeyCode::KeyH => 27,
        KeyCode::KeyI => 28,
        KeyCode::KeyJ => 29,
        KeyCode::KeyK => 30,
        KeyCode::KeyL => 31,
        KeyCode::KeyM => 32,
        KeyCode::KeyN => 33,
        KeyCode::KeyO => 34,
        KeyCode::KeyP => 35,
        KeyCode::KeyQ => 36,
        KeyCode::KeyR => 37,
        KeyCode::KeyS => 38,
        KeyCode::KeyT => 39,
        KeyCode::KeyU => 40,
        KeyCode::KeyV => 41,
        KeyCode::KeyW => 42,
        KeyCode::KeyX => 43,
        KeyCode::KeyY => 44,
        KeyCode::KeyZ => 45,
        KeyCode::Minus => 46,
        KeyCode::Period => 47,
        KeyCode::Quote => 48,
        KeyCode::Semicolon => 49,
        KeyCode::Slash => 50,
        KeyCode::AltLeft => 51,
        KeyCode::AltRight => 52,
        KeyCode::Backspace => 53,
        KeyCode::CapsLock => 54,
        KeyCode::ContextMenu => 55,
        KeyCode::ControlLeft => 56,
        KeyCode::ControlRight => 57,
        KeyCode::Enter => 58,
        KeyCode::SuperLeft => 59,
        KeyCode::SuperRight => 60,
        KeyCode::ShiftLeft => 61,
        KeyCode::ShiftRight => 62,
        KeyCode::Space => 63,
        KeyCode::Tab => 64,
        KeyCode::Convert => 65,
        KeyCode::KanaMode => 66,
        KeyCode::Lang1 => 67,
        KeyCode::Lang2 => 68,
        KeyCode::Lang3 => 69,
        KeyCode::Lang4 => 70,
        KeyCode::Lang5 => 71,
        KeyCode::NonConvert => 72,
        KeyCode::Delete => 73,
        KeyCode::End => 74,
        KeyCode::Help => 75,
        KeyCode::Home => 76,
        KeyCode::Insert => 77,
        KeyCode::PageDown => 78,
        KeyCode::PageUp => 79,
        KeyCode::ArrowDown => 80,
        KeyCode::ArrowLeft => 81,
        KeyCode::ArrowRight => 82,
        KeyCode::ArrowUp => 83,
        KeyCode::NumLock => 84,
        KeyCode::Numpad0 => 85,
        KeyCode::Numpad1 => 86,
        KeyCode::Numpad2 => 87,
        KeyCode::Numpad3 => 88,
        KeyCode::Numpad4 => 89,
        KeyCode::Numpad5 => 90,
        KeyCode::Numpad6 => 91,
        KeyCode::Numpad7 => 92,
        KeyCode::Numpad8 => 93,
        KeyCode::Numpad9 => 94,
        KeyCode::NumpadAdd => 95,
        KeyCode::NumpadBackspace => 96,
        KeyCode::NumpadClear => 97,
        KeyCode::NumpadClearEntry => 98,
        KeyCode::NumpadComma => 99,
        KeyCode::NumpadDecimal => 100,
        KeyCode::NumpadDivide => 101,
        KeyCode::NumpadEnter => 102,
        KeyCode::NumpadEqual => 103,
        KeyCode::NumpadHash => 104,
        KeyCode::NumpadMemoryAdd => 105,
        KeyCode::NumpadMemoryClear => 106,
        KeyCode::NumpadMemoryRecall => 107,
        KeyCode::NumpadMemoryStore => 108,
        KeyCode::NumpadMemorySubtract => 109,
        KeyCode::NumpadMultiply => 110,
        KeyCode::NumpadParenLeft => 111,
        KeyCode::NumpadParenRight => 112,
        KeyCode::NumpadStar => 113,
        KeyCode::NumpadSubtract => 114,
        KeyCode::Escape => 115,
        KeyCode::Fn => 116,
        KeyCode::FnLock => 117,
        KeyCode::PrintScreen => 118,
        KeyCode::ScrollLock => 119,
        KeyCode::Pause => 120,
        KeyCode::BrowserBack => 121,
        KeyCode::BrowserFavorites => 122,
        KeyCode::BrowserForward => 123,
        KeyCode::BrowserHome => 124,
        KeyCode::BrowserRefresh => 125,
        KeyCode::BrowserSearch => 126,
        KeyCode::BrowserStop => 127,
        KeyCode::Eject => 128,
        KeyCode::LaunchApp1 => 129,
        KeyCode::LaunchApp2 => 130,
        KeyCode::LaunchMail => 131,
        KeyCode::MediaPlayPause => 132,
        KeyCode::MediaSelect => 133,
        KeyCode::MediaStop => 134,
        KeyCode::MediaTrackNext => 135,
        KeyCode::MediaTrackPrevious => 136,
        KeyCode::Power => 137,
        KeyCode::Sleep => 138,
        KeyCode::AudioVolumeDown => 139,
        KeyCode::AudioVolumeMute => 140,
        KeyCode::AudioVolumeUp => 141,
        KeyCode::WakeUp => 142,
        KeyCode::Meta => 143,
        KeyCode::Hyper => 144,
        KeyCode::Turbo => 145,
        KeyCode::Abort => 146,
        KeyCode::Resume => 147,
        KeyCode::Suspend => 148,
        KeyCode::Again => 149,
        KeyCode::Copy => 150,
        KeyCode::Cut => 151,
        KeyCode::Find => 152,
        KeyCode::Open => 153,
        KeyCode::Paste => 154,
        KeyCode::Props => 155,
        KeyCode::Select => 156,
        KeyCode::Undo => 157,
        KeyCode::Hiragana => 158,
        KeyCode::Katakana => 159,
        KeyCode::F1 => 160,
        KeyCode::F2 => 161,
        KeyCode::F3 => 162,
        KeyCode::F4 => 163,
        KeyCode::F5 => 164,
        KeyCode::F6 => 165,
        KeyCode::F7 => 166,
        KeyCode::F8 => 167,
        KeyCode::F9 => 168,
        KeyCode::F10 => 169,
        KeyCode::F11 => 170,
        KeyCode::F12 => 171,
        KeyCode::F13 => 172,
        KeyCode::F14 => 173,
        KeyCode::F15 => 174,
        KeyCode::F16 => 175,
        KeyCode::F17 => 176,
        KeyCode::F18 => 177,
        KeyCode::F19 => 178,
        KeyCode::F20 => 179,
        KeyCode::F21 => 180,
        KeyCode::F22 => 181,
        KeyCode::F23 => 182,
        KeyCode::F24 => 183,
        KeyCode::F25 => 184,
        KeyCode::F26 => 185,
        KeyCode::F27 => 186,
        KeyCode::F28 => 187,
        KeyCode::F29 => 188,
        KeyCode::F30 => 189,
        KeyCode::F31 => 190,
        KeyCode::F32 => 191,
        KeyCode::F33 => 192,
        KeyCode::F34 => 193,
        KeyCode::F35 => 194,
        KeyCode::Unidentified(_) => 0,
    }
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
