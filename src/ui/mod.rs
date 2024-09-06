use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::{
    domain::{connection::Connection, drone::Drone},
    io::IOResource,
    misc::{id_tracker::DroneIdTracker, selected_drone::SelectedDrone},
};

pub mod left_panel;
pub mod render_drones;
pub mod right_panel;

pub fn system_drone_ui_left_panel(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut id_tracker: ResMut<DroneIdTracker>,
    mut selected_drone: ResMut<SelectedDrone>,
    mut drones_query: Query<(Entity, &mut Drone)>,
    asset_server: Res<AssetServer>,
) {
    left_panel::show_left_panel(
        &mut commands,
        &mut contexts,
        &mut id_tracker,
        &mut selected_drone,
        &mut drones_query,
        asset_server,
    );
}

pub fn system_drone_ui_right_panel(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut id_tracker: ResMut<DroneIdTracker>,
    mut selected_drone: ResMut<SelectedDrone>,
    mut selected_drones_query: Query<(Entity, &mut Drone, Option<&mut Connection>)>,
    mut io_sender: ResMut<IOResource>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    right_panel::show_right_window(
        &mut commands,
        &mut contexts,
        &mut selected_drone,
        &mut selected_drones_query,
        &mut io_sender,
        &mut camera_query,
    );
}
