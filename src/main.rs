use bevy::{camera_controller::free_camera::FreeCameraPlugin, input::common_conditions::input_toggle_active, prelude::*};


//egui
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

//plugin stuffs ==

mod scene;
use scene::ScenePlugin;

mod code;
use code::CodePlugin;


fn main() {
    App::new()

    //world
    .insert_resource(ClearColor(Color::BLACK))

    //window ==
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Tart2026".into(),
            ..default()
        }),
        ..default()
    }),)

    //egui
    .add_plugins(EguiPlugin::default())
    .add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
    )

    //bevy plugins
    .add_plugins(FreeCameraPlugin)

    //plugins
    .add_plugins(ScenePlugin)

    .run();
}