use std::collections::HashMap;

use bevy::prelude::Resource;
use blitz_common::PlayerInput;

#[derive(Debug, Default)]
pub struct PlayerData {
    pub input: PlayerInput,
}

#[derive(Debug, Default, Resource)]
pub struct Lobby {
    pub players: HashMap<u64, PlayerData>,
}
