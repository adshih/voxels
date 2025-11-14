mod camera;
mod debug;
mod network;
mod player;
mod world;

use bevy::asset::AssetMetaCheck;
use camera::CameraPlugin;
use debug::DebugPlugin;
use network::NetworkPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
enum Systems {
    Input,
    Movement,
    PostMovement,
}

#[derive(Default, Debug, Resource)]
struct Settings {
    multiplayer: bool,
}

fn main() {
    let settings = Settings { multiplayer: true };

    let mut app = App::new();

    app.configure_sets(
        Update,
        (
            Systems::Input.run_if(is_cursor_locked),
            Systems::Movement.after(Systems::Input),
            Systems::PostMovement.after(Systems::Movement),
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
        Name::new("Light"),
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
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
