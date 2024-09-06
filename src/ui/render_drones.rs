use bevy::{prelude::*, render::mesh::PrimitiveTopology};

use crate::{
    domain::{coordinates::COORDS_ZOOM, drone::Drone, mission::Mission},
    misc::selected_drone::SelectedDrone,
};

#[derive(Component)]
pub struct Temporary;

pub fn system_render_drones(
    mut drones_query: Query<(Entity, &Drone, &mut Transform, Option<&Mission>)>,
    mut commands: Commands,
    selected_drone: Res<SelectedDrone>,
    asset_server: Res<AssetServer>,
) {
    for (entity, drone, mut trans, mission_opt) in drones_query.iter_mut() {
        trans.translation.x = drone.coordinates.longitude * COORDS_ZOOM;
        trans.translation.y = drone.coordinates.latitude * COORDS_ZOOM;

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
                            target.longitude * COORDS_ZOOM,
                            target.latitude * COORDS_ZOOM,
                            0.0,
                        ),
                        scale: Vec3::new(0.007, 0.007, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Temporary);
        }
    }
}

pub fn system_despawn_entities(mut commands: Commands, query: Query<Entity, With<Temporary>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
