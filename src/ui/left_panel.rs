use crate::{
    domain::{
        coordinates::Coordinates,
        drone::{Drone, DroneState},
    },
    misc::{id_tracker::DroneIdTracker, selected_drone::SelectedDrone},
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn show_left_panel(
    commands: &mut Commands,
    contexts: &mut EguiContexts,
    id_tracker: &mut ResMut<DroneIdTracker>,
    selected_drone: &mut ResMut<SelectedDrone>,
    drones_query: &mut Query<(Entity, &mut Drone)>,
    asset_server: Res<AssetServer>,
) {
    egui::SidePanel::left("drone_control_panel")
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Drone Control");

            render_top_buttons(
                ui,
                commands,
                id_tracker,
                drones_query,
                selected_drone,
                asset_server,
            );
            ui.separator();
            render_drone_list(ui, drones_query, selected_drone);
        });
}

fn render_top_buttons(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    id_tracker: &mut ResMut<DroneIdTracker>,
    drones_query: &mut Query<(Entity, &mut Drone)>,
    selected_drone: &mut ResMut<SelectedDrone>,
    asset_server: Res<AssetServer>,
) {
    ui.horizontal(|ui| {
        if ui.button("Create Drone").clicked() {
            create_new_drone(commands, id_tracker, asset_server);
        }

        if ui.button("Delete All Drones").clicked() {
            delete_all_drones(commands, drones_query, selected_drone);
        }
    });
}

fn create_new_drone(
    commands: &mut Commands,
    id_tracker: &mut ResMut<DroneIdTracker>,
    asset_server: Res<AssetServer>,
) {
    let next_id = id_tracker.increment();
    commands
        .spawn(Drone {
            agent_id: next_id,
            state: DroneState::Offline,
            coordinates: Coordinates {
                longitude: -9.114488884434095,
                latitude: 38.75600095957655,
            },
        })
        .insert(SpriteBundle {
            texture: asset_server.load("drone.png"),
            transform: Transform {
                scale: Vec3::new(0.003, 0.003, 1.0),
                ..Default::default()
            },
            ..Default::default()
        });
}

fn delete_all_drones(
    commands: &mut Commands,
    drones_query: &mut Query<(Entity, &mut Drone)>,
    selected_drone: &mut ResMut<SelectedDrone>,
) {
    for (entity, _) in drones_query.iter() {
        commands.entity(entity).despawn();
    }
    selected_drone.entity = None; // Deselect any selected drone
}

fn render_drone_list(
    ui: &mut egui::Ui,
    drones_query: &mut Query<(Entity, &mut Drone)>,
    selected_drone: &mut ResMut<SelectedDrone>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (entity, drone) in drones_query.iter_mut() {
            let drone_label = format!("Drone ID: {}, State: {}", drone.agent_id, drone.state);

            if ui.button(&drone_label).clicked() {
                selected_drone.entity = Some(entity); // Select the drone
            }
        }
    });
}
