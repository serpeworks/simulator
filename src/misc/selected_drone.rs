use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct SelectedDrone {
    pub entity: Option<Entity>,
}
