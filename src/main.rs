use bevy::prelude::*;
use domain::{coordinates::COORDS_ZOOM, mission::{
    system_mission_update_coordinates, system_mission_update_sender, system_mission_updater,
    MissionUpdateTimer,
}};
use io::{run_io, IOResource};
use misc::{
    heartbeat::{system_heartbeat, HeartbeatTimer},
    id_tracker::DroneIdTracker,
    selected_drone::SelectedDrone,
};
use tokio_util::sync::CancellationToken;
// use ui::{
//     render_drones::{system_despawn_entities, system_render_drones, ZOOM},
//     system_drone_ui_left_panel, system_drone_ui_right_panel,
// };

pub mod domain;
pub mod io;
pub mod misc;
pub mod ui;

mod mavlink {
    include!(concat!(env!("OUT_DIR"), "/mavlink/mod.rs"));
}

const GUI_SCALE_FACTOR: f32 = 1.5;

use bevy::input::keyboard::KeyCode;

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel(1000);

    let token = CancellationToken::new();
    let io_token = token.clone();

    tokio::spawn(async move {
        run_io(rx, io_token).await;
    });

    App::new()
        .insert_resource(DroneIdTracker::default())
        .insert_resource(SelectedDrone::default())
        // .insert_resource(EguiSettings {
        //     scale_factor: GUI_SCALE_FACTOR,
        //     ..Default::default()
        // })
        .insert_resource(IOResource { sender: tx })
        .insert_resource(HeartbeatTimer::default())
        .insert_resource(MissionUpdateTimer::default())
        .add_plugins(DefaultPlugins)
        // .add_plugins(EguiPlugin)
        .add_systems(Startup, system_setup)
        // .add_systems(Update, system_drone_ui_left_panel)
        // .add_systems(Update, system_drone_ui_right_panel)
        // .add_systems(Update, system_despawn_entities)
        // .add_systems(Update, system_render_drones)
        .add_systems(Update, system_mission_updater)
        .add_systems(Update, system_mission_update_sender)
        .add_systems(Update, system_mission_update_coordinates)
        .add_systems(Update, system_heartbeat)
        .add_systems(Update, camera_movement)
        .run();

    token.cancel();
}

fn system_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(-9.1144_888 * COORDS_ZOOM, 38.756 * COORDS_ZOOM, 1.0),
        projection: OrthographicProjection {
            scale: 1.0 / 35.0,
            ..default()
        }
        .into(),
        ..default()
    });
}

fn camera_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    let mut camera_transform = query.single_mut();

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let speed = 25.0;
    camera_transform.translation += direction * speed * 0.01;
}
