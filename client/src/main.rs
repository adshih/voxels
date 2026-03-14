mod camera;
mod connection;
mod debug;
mod player;
mod world;

use std::{env, f32::consts::PI};

use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use camera::CameraPlugin;
use connection::NetworkPlugin;
use debug::DebugPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
enum Systems {
    Input,
    Movement,
    PostMovement,
    Network,
    Chunk,
    Mesh,
}

#[derive(Default, Debug, Resource)]
pub struct Settings {
    addr: Option<String>,
    name: String,
}

impl Settings {
    pub fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();

        let addr = args
            .iter()
            .position(|a| a == "--connect" || a == "-c")
            .and_then(|i| args.get(i + 1))
            .cloned();

        let name = args
            .iter()
            .position(|a| a == "--name" || a == "-n")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| "Player".to_string());

        Self { addr, name }
    }
}

fn main() {
    let settings = Settings::from_args();
    let mut app = App::new();

    app.configure_sets(
        Update,
        (
            Systems::Input.run_if(is_cursor_locked),
            Systems::Movement.after(Systems::Input),
            Systems::PostMovement.after(Systems::Movement),
            Systems::Network.after(Systems::PostMovement),
            Systems::Chunk.after(Systems::Network),
            Systems::Mesh.after(Systems::Network),
        ),
    )
    .add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
        CameraPlugin,
        DebugPlugin,
        PlayerPlugin,
        WorldPlugin,
        NetworkPlugin,
    ))
    .insert_resource(settings)
    .add_systems(Startup, setup)
    .add_systems(Update, toggle_cursor_lock);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
    ));
}

fn is_cursor_locked(primary_cursor_options: Single<&CursorOptions, With<PrimaryWindow>>) -> bool {
    primary_cursor_options.grab_mode == CursorGrabMode::Locked
}

fn toggle_cursor_lock(
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match primary_cursor_options.grab_mode {
            CursorGrabMode::None => {
                primary_cursor_options.grab_mode = CursorGrabMode::Locked;
                primary_cursor_options.visible = false;
            }
            CursorGrabMode::Locked => {
                primary_cursor_options.grab_mode = CursorGrabMode::None;
                primary_cursor_options.visible = true;
            }
            _ => (),
        }
    }
}
