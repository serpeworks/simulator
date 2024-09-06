use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct DroneIdTracker {
    next_id: u32,
}

impl DroneIdTracker {
    pub fn increment(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }
}
