use std::collections::HashMap;

use bevy::prelude::{Entity, Resource};

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}
