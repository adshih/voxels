use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::player::LocalPlayer;
use crate::world::MAX_MESH_TASKS;
use crate::world::chunk::ChunkLoadQueue;
use crate::world::mesh::{MeshReady, MeshTask, NeedsMesh};

#[derive(Component)]
struct DebugPanel;

#[derive(Component)]
struct PerformanceText;

#[derive(Component)]
struct PlayerText;

#[derive(Component)]
struct CameraText;

#[derive(Component)]
struct ChunkText;

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
                debug_chunks,
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
    mut visibility: Single<&mut Visibility, With<DebugPanel>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        debug_state.enabled = !debug_state.enabled;

        **visibility = if debug_state.enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
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
            parent.spawn((DebugTextBundle::default(), ChunkText));
        });
}

fn debug_performance(
    mut text: Single<&mut Text, With<PerformanceText>>,
    diagnostics: Res<DiagnosticsStore>,
) {
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

    text.0 = perf_info;
}

fn debug_player_position(
    player_transform: Single<&Transform, With<LocalPlayer>>,
    mut text: Single<&mut Text, With<PlayerText>>,
) {
    let pos = player_transform.translation;
    text.0 = format!("XYZ: {:.1} / {:.1} / {:.1}", pos.x, pos.y, pos.z);
}

fn debug_camera_info(
    camera_transform: Single<&Transform, With<Camera>>,
    mut text: Single<&mut Text, With<CameraText>>,
) {
    let forward = camera_transform.forward();
    let rotation = camera_transform.rotation.to_euler(EulerRot::YXZ);

    text.0 = format!(
        "Facing: {:.2} / {:.2} / {:.2}\nRotation: {:.1} / {:.1}",
        forward.x,
        forward.y,
        forward.z,
        rotation.1.to_degrees(), // pitch
        rotation.0.to_degrees()  // yaw
    );
}

fn debug_chunks(
    needs_mesh: Query<(), With<NeedsMesh>>,
    mesh_tasks: Query<(), With<MeshTask>>,
    mesh_ready: Query<(), With<MeshReady>>,
    chunk_load_queue: Res<ChunkLoadQueue>,
    mut text: Single<&mut Text, With<ChunkText>>,
) {
    text.0 = format!(
        r#"
Chunks:
    ChunkLoadQueue: {},
    NeedsMesh: {},
    MeshTask: {}/{}
    MeshReady: {}
        "#,
        chunk_load_queue.0.len(),
        needs_mesh.iter().count(),
        mesh_tasks.iter().count(),
        MAX_MESH_TASKS,
        mesh_ready.iter().count(),
    );
}
