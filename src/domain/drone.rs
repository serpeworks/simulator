use bevy::prelude::*;
use core::fmt;

use super::coordinates::Coordinates;

#[derive(Debug, PartialEq)]
pub enum DroneState {
    Offline,
    Online,
}

impl fmt::Display for DroneState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DroneState::Offline => write!(f, "Offline"),
            DroneState::Online => write!(f, "Online"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connected,
    Broken,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Broken => write!(f, "Broken"),
        }
    }
}

#[derive(Debug, Component)]
pub struct Drone {
    pub agent_id: u32,
    pub state: DroneState,
    pub coordinates: Coordinates,
}

#[derive(Bundle)]
pub struct DroneBundle {
    drone: Drone,
}
