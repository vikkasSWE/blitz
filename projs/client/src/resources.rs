use std::collections::HashMap;

use bevy::prelude::{Entity, Handle, Image, Resource};

pub const PLAYER_SPRITE: &str = "../../../assets/player_b_01.png";
pub const PLAYER_LASER_SPRITE: &str = "../../../assets/laser_a_01.png";

#[derive(Resource)]
pub struct Textures {
    pub player: Handle<Image>,
    pub player_laser: Handle<Image>,
}

#[derive(Resource)]
pub struct WinSize {
    #[allow(unused)]
    pub width: f32,
    pub height: f32,
}

#[derive(Default, Resource)]
pub struct NetworkMapping(pub HashMap<Entity, Entity>);

#[derive(Debug)]
pub struct PlayerInfo {
    pub client_entity: Entity,
    pub server_entity: Entity,
}

#[derive(Debug, Default, Resource)]
pub struct ClientLobby {
    pub players: HashMap<u64, PlayerInfo>,
}
