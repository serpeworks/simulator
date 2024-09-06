use std::time::{Duration, Instant};

use bevy::prelude::*;

use crate::{
    domain::{connection::Connection, drone::Drone},
    mavlink::dialects::serpe_dialect::messages::Heartbeat,
};

#[derive(Resource)]
pub struct HeartbeatTimer {
    last_time: Instant,
}

impl Default for HeartbeatTimer {
    fn default() -> Self {
        HeartbeatTimer {
            last_time: Instant::now(),
        }
    }
}

pub fn system_heartbeat(
    mut heartbeat_timer: ResMut<HeartbeatTimer>,
    mut connection_query: Query<(&Drone, &mut Connection)>,
) {
    let current_time = Instant::now();

    if current_time.duration_since(heartbeat_timer.last_time) >= Duration::from_secs(1) {
        for (drone, connection) in connection_query.iter_mut() {

            let _ = connection
                .sender
                .try_send(crate::mavlink::dialects::SerpeDialect::Heartbeat(
                    Heartbeat {
                        latitude: drone.coordinates.latitude,
                        longitude: drone.coordinates.longitude,
                    },
                ));
        }

        heartbeat_timer.last_time = current_time;
    }
}
