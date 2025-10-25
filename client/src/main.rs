mod camera;
mod debug;
mod network;
mod player;
mod world;

use bevy::asset::AssetMetaCheck;
use bevy_fix_cursor_unlock_web::FixPointerUnlockPlugin;
use camera::CameraPlugin;
use debug::DebugPlugin;
use network::NetworkPlugin;
use player::PlayerPlugin;
use world::WorldPlugin;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

#[derive(SystemSet, Debug, Clone, Hash, Eq, PartialEq)]
enum Systems {
    Input,
    Movement,
    PostMovement,
}

#[derive(Default, Debug, Resource)]
struct Settings {
    auto_lock_cursor: bool,
    multiplayer: bool,
}

fn main() {
    let settings = Settings {
        auto_lock_cursor: false,
        multiplayer: true,
    };

    let cursor_options = CursorOptions {
        grab_mode: if settings.auto_lock_cursor {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        },
        visible: !settings.auto_lock_cursor,
        ..default()
    };

    let mut app = App::new();

    if settings.multiplayer {
        app.add_plugins(NetworkPlugin);
    }

    app.configure_sets(
        Update,
        (
            Systems::Input.run_if(cursor_locked),
            Systems::Movement.after(Systems::Input),
            Systems::PostMovement.after(Systems::Movement),
        ),
    )
    .add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    cursor_options,
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
        FixPointerUnlockPlugin,
    ))
    .insert_resource(settings)
    .add_systems(Startup, setup)
    .add_systems(Update, toggle_cursor_lock);

    app.run();
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>, settings: Res<Settings>) {
    if settings.auto_lock_cursor {
        let mut window = window.single_mut().expect("Could not find window");
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }

    commands.spawn((
        Name::new("Light"),
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

fn cursor_locked(window_query: Query<&Window>) -> bool {
    if let Ok(window) = window_query.single() {
        window.cursor_options.grab_mode == CursorGrabMode::Locked
    } else {
        false
    }
}

fn toggle_cursor_lock(mut window: Query<&mut Window>, keyboard: Res<ButtonInput<KeyCode>>) {
    let mut window = window.single_mut().expect("Could not find window");
    let cursor_options = &mut window.cursor_options;

    if keyboard.just_pressed(KeyCode::Escape) {
        match cursor_options.grab_mode {
            CursorGrabMode::None => {
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
            CursorGrabMode::Locked => {
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
            _ => (),
        }
    }
}
