use bevy::{prelude::*, render::mesh::PrimitiveTopology};

use crate::{
    domain::{drone::Drone, mission::Mission},
    misc::selected_drone::SelectedDrone,
};

pub const ZOOM: f32 = 1000.0;

#[derive(Component)]
pub struct Temporary;

pub fn system_render_drones(
    mut drones_query: Query<(Entity, &Drone, &mut Transform, Option<&Mission>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
    selected_drone: Res<SelectedDrone>,
    asset_server: Res<AssetServer>,
) {
    for (entity, drone, mut trans, mission_opt) in drones_query.iter_mut() {
        trans.translation.x = drone.coordinates.longitude * ZOOM;
        trans.translation.y = drone.coordinates.latitude * ZOOM;

        if let Some(selected) = selected_drone.entity {
            if selected != entity {
                continue;
            }
        } else {
            continue;
        }

        if let Some(mission) = mission_opt {
            let target = mission.target;
            commands
                .spawn(SpriteBundle {
                    texture: asset_server.load("target.png"),
                    transform: Transform {
                        translation: Vec3::new(
                            target.longitude * ZOOM,
                            target.latitude * ZOOM,
                            0.0,
                        ),
                        scale: Vec3::new(0.007, 0.007, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Temporary);

            let waypoints = mission.waypoints.clone();
            if waypoints.len() > 1 {
                let mut points = Vec::new();
                for waypoint in waypoints {
                    points.push(Vec3::new(
                        waypoint.longitude * ZOOM,
                        waypoint.latitude * ZOOM,
                        0.0,
                    ));
                }

                commands
                    .spawn(MaterialMeshBundle {
                        mesh: meshes.add(LineStrip { points: todo!() }),
                        transform: Transform::default(),
                        ..default()
                    })
                    .insert(Temporary);
            }
        }
    }
}

pub fn system_despawn_entities(mut commands: Commands, query: Query<Entity, With<Temporary>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
