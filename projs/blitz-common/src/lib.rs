use bevy::{log, prelude::*};
use bevy_renet::renet::RenetError;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Default, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
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
