use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::player::Player;
use crate::world::ChunkManager;

#[derive(Component)]
struct DebugPanel;

#[derive(Component)]
struct PerformanceText;

#[derive(Component)]
struct PlayerText;

#[derive(Component)]
struct CameraText;

#[derive(Component)]
struct ChunkManagerText;

#[derive(Bundle)]
struct DebugTextBundle {
    text: Text,
    font: TextFont,
    color: TextColor,
}

impl Default for DebugTextBundle {
    fn default() -> Self {
        Self {
            text: Text::new(""),
            font: TextFont {
                font_size: 14.0,
                ..default()
            },
            color: TextColor(Color::WHITE),
        }
    }
}

#[derive(Resource, Default)]
struct DebugState {
    enabled: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin {
            smoothing_factor: 0.5,
            ..default()
        })
        .add_systems(Startup, setup_debug_ui)
        .add_systems(
            Update,
            (
                debug_performance,
                debug_player_position,
                debug_camera_info,
                debug_chunk_manager,
            )
                .run_if(is_debug),
        )
        .add_systems(Update, toggle_debug);
    }
}

fn is_debug(debug_state: Res<DebugState>) -> bool {
    debug_state.enabled
}

fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<DebugState>,
    mut debug_panel: Query<&mut Visibility, With<DebugPanel>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        debug_state.enabled = !debug_state.enabled;

        if let Ok(mut visibility) = debug_panel.single_mut() {
            *visibility = if debug_state.enabled {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn setup_debug_ui(mut commands: Commands) {
    commands.insert_resource(DebugState { enabled: true });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(4.0),
                left: Val::Px(4.0),
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(1.0),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.3)),
            DebugPanel,
        ))
        .with_children(|parent| {
            parent.spawn((DebugTextBundle::default(), PerformanceText));
            parent.spawn((DebugTextBundle::default(), PlayerText));
            parent.spawn((DebugTextBundle::default(), CameraText));
            parent.spawn((DebugTextBundle::default(), ChunkManagerText));
        });
}

fn debug_performance(
    mut debug_text: Query<&mut Text, With<PerformanceText>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if let Ok(mut text) = debug_text.single_mut() {
        let mut perf_info = String::new();

        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
            && let Some(value) = fps.smoothed()
        {
            perf_info.push_str(&format!("{:.0} fps", value));

            if let Some(frame_time) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                && let Some(ft_value) = frame_time.smoothed()
            {
                perf_info.push_str(&format!(" ({:.1}ms)", ft_value));
            }
        }

        **text = perf_info;
    }
}

fn debug_player_position(
    player_query: Query<&Transform, With<Player>>,
    mut debug_text: Query<&mut Text, With<PlayerText>>,
) {
    if let (Ok(player_transform), Ok(mut text)) = (player_query.single(), debug_text.single_mut()) {
        let pos = player_transform.translation;

        **text = format!("XYZ: {:.1} / {:.1} / {:.1}", pos.x, pos.y, pos.z);
    }
}

fn debug_camera_info(
    camera_query: Query<&Transform, With<Camera>>,
    mut debug_text: Query<&mut Text, With<CameraText>>,
) {
    if let (Ok(camera_transform), Ok(mut text)) = (camera_query.single(), debug_text.single_mut()) {
        let forward = camera_transform.forward();
        let rotation = camera_transform.rotation.to_euler(EulerRot::YXZ);

        **text = format!(
            "Facing: {:.2} / {:.2} / {:.2}\nRotation: {:.1} / {:.1}",
            forward.x,
            forward.y,
            forward.z,
            rotation.1.to_degrees(), // pitch
            rotation.0.to_degrees()  // yaw
        );
    }
}

fn debug_chunk_manager(
    chunk_manager: Res<ChunkManager>,
    mut debug_text: Query<&mut Text, With<ChunkManagerText>>,
) {
    if let Ok(mut text) = debug_text.single_mut() {
        **text = format!(
            "Chunk Manager:\nPending Operations: {}\nLoaded: {}",
            chunk_manager.pending_ops.len(),
            chunk_manager.loaded_chunks.len(),
        );
    }
}
