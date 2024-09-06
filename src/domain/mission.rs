use bevy::prelude::*;

use super::coordinates::Coordinates;

#[derive(Debug, Component)]
pub struct Mission {
    pub mission_id: u32,
    pub target: Coordinates,
}
