use std::time::{Duration, Instant};

use bevy::prelude::*;

use crate::{
    mavlink::dialects::{
        serpe_dialect::messages::{MissionAccept, MissionUpdate},
        SerpeDialect,
    },
};

use super::{connection::Connection, coordinates::{Coordinates, COORDS_ZOOM}, drone::Drone};

const DRONE_SPEED: f32 = 0.001;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MissionState {
    AwaitingAcceptAck,
    Ongoing,
    AwaitingFinishedAck,
}

#[derive(Clone, Debug, Component)]
pub struct Mission {
    pub state: MissionState,
    pub target: Coordinates,
    pub waypoints: Vec<Coordinates>,
}

pub fn system_mission_updater(
    mut drones_query: Query<(Entity, &Drone, &mut Connection, Option<&mut Mission>)>,
    mut commands: Commands,
) {
    for (entity, _, mut connection, mut mission_opt) in drones_query.iter_mut() {
        while let Ok(message) = connection.receiver.try_recv() {
            match message {
                SerpeDialect::MissionRequest(msg) => {
                    if mission_opt.is_some() {
                        // ignore if it already has a mission;
                        continue;
                    } else {
                        let _ = connection
                            .sender
                            .try_send(SerpeDialect::MissionAccept(MissionAccept {}));

                        commands.entity(entity).insert(Mission {
                            state: MissionState::AwaitingAcceptAck,
                            target: Coordinates {
                                latitude: msg.target_latitude,
                                longitude: msg.target_longitude,
                            },
                            waypoints: vec![], // TODO
                        });
                    }
                }
                crate::mavlink::dialects::SerpeDialect::MissionAcceptAck(msg) => {
                    match mission_opt {
                        Some(ref mut mission)
                            if mission.state == MissionState::AwaitingAcceptAck =>
                        {
                            mission.state = MissionState::Ongoing;
                        }
                        Some(_) => {
                            println!("received an awating accept ack on already going mission");
                        }
                        None => {
                            println!(
                                "received an awating accept ack a drone with no requested mission"
                            );
                        }
                    }
                }
                crate::mavlink::dialects::SerpeDialect::MissionFinishedAck(msg) => {
                    commands.entity(entity).remove::<Mission>();
                }
                _ => {}
            }
        }
    }
}

#[derive(Resource)]
pub struct MissionUpdateTimer {
    last_time: Instant,
}

impl Default for MissionUpdateTimer {
    fn default() -> Self {
        Self {
            last_time: Instant::now(),
        }
    }
}

pub fn system_mission_update_sender(
    mut mission_update_timer: ResMut<MissionUpdateTimer>,
    mut connection_query: Query<(&Drone, &mut Connection, Option<&Mission>)>,
) {
    let current_time = Instant::now();

    if current_time.duration_since(mission_update_timer.last_time) >= Duration::from_secs(1) {
        for (drone, connection, mission_opt) in connection_query.iter_mut() {
            match mission_opt {
                Some(mission) => {
                    if mission.state != MissionState::Ongoing {
                        continue;
                    }
                }
                None => continue,
            }

            let _ =
                connection
                    .sender
                    .try_send(crate::mavlink::dialects::SerpeDialect::MissionUpdate(
                        MissionUpdate {
                            current_latitude: drone.coordinates.latitude * COORDS_ZOOM,
                            current_longitude: drone.coordinates.longitude * COORDS_ZOOM,
                        },
                    ));
        }

        mission_update_timer.last_time = current_time;
    }
}

pub fn system_mission_update_coordinates(
    time: Res<Time>,
    mut connection_query: Query<(&mut Drone, &mut Mission, &mut Connection)>,
) {
    for (mut drone, mut mission, connection) in connection_query.iter_mut() {
        if mission.state != MissionState::Ongoing {
            continue;
        }

        let target = mission.target;

        let step = DRONE_SPEED * time.delta_seconds();

        let current_latitude = drone.coordinates.latitude;
        let current_longitude = drone.coordinates.longitude;

        let delta_latitude = target.latitude - current_latitude;
        let delta_longitude = target.longitude - current_longitude;

        if delta_latitude.abs() > step {
            drone.coordinates.latitude += delta_latitude.signum() * step;
        } else {
            drone.coordinates.latitude = target.latitude;
        }

        if delta_longitude.abs() > step {
            drone.coordinates.longitude += delta_longitude.signum() * step;
        } else {
            drone.coordinates.longitude = target.longitude;
        }

        if (drone.coordinates.latitude - target.latitude).abs() < step
            && (drone.coordinates.longitude - target.longitude).abs() < step
        {
            mission.state = MissionState::AwaitingFinishedAck;
            let _ = connection.sender.try_send(SerpeDialect::MissionFinished(
                crate::mavlink::dialects::serpe_dialect::messages::MissionFinished {},
            ));
        }
    }
}
