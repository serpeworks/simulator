use bevy::prelude::*;

use crate::domain::drone::Drone;

const zoom : f32 = 1000.0;

pub fn system_render_drones(mut drones_query: Query<(Entity, &mut Drone, &mut Transform)>) {
    for (_, mut drone, mut trans) in drones_query.iter_mut() {
        trans.translation.x = drone.coordinates.longitude * zoom;
        trans.translation.y = drone.coordinates.latitude * zoom;
    }
}
