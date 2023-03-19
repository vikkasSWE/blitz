use std::collections::HashMap;

use bevy::{log, prelude::*};
use bevy_renet::renet::RenetError;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: u64 = 7;
pub const PLAYER_MOVE_SPEED: f32 = 200.0;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, Component, Resource, PartialEq)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub mouse: Vec2,
}

#[derive(Component, Default, Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub entity: Option<Entity>,
    pub input: PlayerInput,
    pub transform: [f32; 2],
}

#[derive(Debug, Default, Serialize, Deserialize, Resource)]
pub struct Lobby {
    pub players: HashMap<u64, Player>,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
}

/// If any error is found we just panic
pub fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        match e {
            RenetError::Netcode(e) => log::error!("{e}"),
            RenetError::Rechannel(e) => log::error!("{e}"),
            RenetError::IO(e) => log::error!("{e}"),
        }
        panic!();
    }
}
