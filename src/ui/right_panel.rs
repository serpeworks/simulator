use std::time::Duration;

use crate::{
    domain::{
        connection::Connection,
        coordinates::Coordinates,
        drone::{ConnectionState, Drone, DroneState},
    },
    io::{IOMessage, IOResource},
    mavlink::dialects::serpe_dialect::{self, messages::Unregister},
    misc::selected_drone::SelectedDrone,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

const COORDINATES_DRAG_SPEED: f64 = 0.00001;

pub fn show_right_window(
    commands: &mut Commands,
    contexts: &mut EguiContexts,
    selected_drone: &mut ResMut<SelectedDrone>,
    drones_query: &mut Query<(Entity, &mut Drone, Option<&mut Connection>)>,
    io_sender: &mut ResMut<IOResource>,
    camera_query: &mut Query<&mut Transform, With<Camera2d>>,
) {
    if let Some(selected_entity) = selected_drone.entity {
        if let Ok((entity, mut drone, connection)) = drones_query.get_mut(selected_entity) {
            show_drone_details_window(
                commands,
                contexts,
                entity,
                &mut drone,
                connection,
                selected_drone,
                io_sender,
                camera_query,
            );
        }
    }
}

fn show_drone_details_window(
    commands: &mut Commands,
    contexts: &mut EguiContexts,
    entity: Entity,
    drone: &mut Drone,
    connection: Option<Mut<Connection>>,
    selected_drone: &mut ResMut<SelectedDrone>,
    io_sender: &mut ResMut<IOResource>,
    camera_query: &mut Query<&mut Transform, With<Camera2d>>,
) {
    let screen_width = contexts.ctx_mut().screen_rect().max.x;
    let window_pos = egui::pos2(screen_width - 310.0, 100.0);

    let mut is_open = true;
    egui::Window::new("Drone Details")
        .fixed_size((300.0, 200.0)) // Make the window always expanded with a fixed size
        .default_pos(window_pos) // Position the window on the right
        .open(&mut is_open) // Handle the "X" close button
        .show(contexts.ctx_mut(), |ui| {
            render_drone_details(
                commands,
                ui,
                entity,
                drone,
                connection,
                io_sender,
                camera_query,
            );
        });

    if !is_open {
        selected_drone.entity = None; // Close the details window if "X" is clicked
    }
}

fn render_drone_details(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    entity: Entity,
    drone: &mut Drone,
    connection: Option<Mut<Connection>>,
    io_sender: &mut ResMut<IOResource>,
    camera_query: &mut Query<&mut Transform, With<Camera2d>>,
) {
    render_drone_header(ui, drone);
    ui.separator();
    render_drone_state(commands, ui, entity, drone, connection, io_sender);
    ui.separator();
    render_drone_coordinates(ui, &mut drone.coordinates, camera_query);
}

fn render_drone_header(ui: &mut egui::Ui, drone: &Drone) {
    ui.heading(format!("Agent ID: {}", drone.agent_id));
}

fn render_drone_state(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    entity: Entity,
    drone: &mut Drone,
    connection: Option<Mut<Connection>>,
    io_sender: &mut ResMut<IOResource>,
) {
    ui.label(format!("State: {}", drone.state));

    if drone.state == DroneState::Offline {
        if ui.button("Turn On").clicked() {
            drone.state = DroneState::Online;
        }
    } else if drone.state == DroneState::Online && connection.is_none() {
        if ui.button("Turn Off").clicked() {
            drone.state = DroneState::Offline;
        }

        if ui.button("Connect").clicked() {
            match create_connection(drone.agent_id, io_sender, drone.coordinates) {
                Ok(connection) => {
                    commands.entity(entity).insert(connection);
                }
                Err(_) => {
                    println!("Unsuccessful Connection");
                }
            }
        }
    }

    if drone.state == DroneState::Online {
        if let Some(mut connection) = connection {
            let connection_status = if is_connection_broken(&connection) {
                ConnectionState::Broken
            } else {
                ConnectionState::Connected
            };
            ui.label(format!("Connection Status: {}", connection_status));

            ui.label(format!("System ID: {}", connection.system_id));

            if ui.button("Disconnect").clicked() {
                on_disconnect(commands, entity, &mut connection);
            }
        } else {
            ui.label("No connection established.");
        }
    }
}

fn on_disconnect(commands: &mut Commands, entity: Entity, connection: &mut Connection) {
    let _ = connection
        .sender
        .try_send(serpe_dialect::SerpeDialect::Unregister(Unregister {}));

    std::thread::sleep(Duration::from_millis(100));
    // TODO: wait for unregister ack

    commands.entity(entity).remove::<Connection>();
    connection.receiver.close();
}

fn create_connection(
    agent_id: u32,
    io_sender: &mut ResMut<IOResource>,
    coordinates: Coordinates,
) -> Result<Connection, ()> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let message = IOMessage::CreateConnection {
        agent_id,
        tx,
        coordinates,
    };

    if let Err(_) = io_sender.sender.try_send(message) {
        return Err(());
    }

    return match rx.blocking_recv() {
        Ok(connection) => Ok(connection),
        Err(_) => Err(()),
    };
}

fn is_connection_broken(connection: &Connection) -> bool {
    connection.receiver.is_closed()
}

fn render_drone_coordinates(
    ui: &mut egui::Ui,
    coordinates: &mut Coordinates,
    camera_query: &mut Query<&mut Transform, With<Camera2d>>,
) {
    ui.label("Physical Properties");

    ui.horizontal(|ui| {
        ui.label("Latitude:");
        ui.add(egui::DragValue::new(&mut coordinates.latitude).speed(COORDINATES_DRAG_SPEED));

        ui.separator();

        ui.label("Longitude:");
        ui.add(egui::DragValue::new(&mut coordinates.longitude).speed(COORDINATES_DRAG_SPEED));
    });

    ui.separator();

    if ui.button("Center").clicked() {
        let mut camera = camera_query.single_mut();
        camera.translation.x = coordinates.longitude * 1000.0;
        camera.translation.y = coordinates.latitude * 1000.0;
    }
}
