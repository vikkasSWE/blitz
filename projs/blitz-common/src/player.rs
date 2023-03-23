use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const PLAYER_MOVE_SPEED: f32 = 200.0;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, Component, Resource, PartialEq)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub mouse: Vec2,
    pub space: bool,
}

#[derive(Component, Default, Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: u64,
}
#[derive(Component)]
pub struct FromPlayer {
    pub entity: Entity,
}
