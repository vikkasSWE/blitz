use std::{collections::HashMap, time::Duration};

use bevy::{log, prelude::*};
use bevy_renet::renet::{
    ChannelConfig, ReliableChannelConfig, RenetConnectionConfig, RenetError,
    UnreliableChannelConfig,
};
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
    pub space: bool,
}

#[derive(Component, Default, Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug, Component, Default)]
pub struct Projectile {
    pub duration: Timer,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub transforms: Vec<Transform>,
}

#[derive(Debug, Default, Serialize, Deserialize, Resource)]
pub struct Lobby {
    pub players: HashMap<u64, Entity>,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessage {
    PlayerConnected {
        id: u64,
        entity: Entity,
    },
    PlayerDisconnected {
        id: u64,
    },
    SpawnProjectile {
        entity: Entity,
        transform: Vec2,
        rotation: Quat,
    },
    DespawnProjectile {
        entity: Entity,
    },
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum PlayerCommand {
    BasicAttack { cast_at: Vec2 },
}

pub enum ClientChannel {
    Input,
    Command,
}

pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
            ClientChannel::Input => 1,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ReliableChannelConfig {
                channel_id: Self::Input.into(),
                message_resend_time: Duration::ZERO,
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Command.into(),
                message_resend_time: Duration::ZERO,
                ..Default::default()
            }
            .into(),
        ]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            UnreliableChannelConfig {
                channel_id: Self::NetworkedEntities.into(),
                sequenced: true, // We don't care about old positions
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::ServerMessages.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn client_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ClientChannel::channels_config(),
        receive_channels_config: ServerChannel::channels_config(),
        ..Default::default()
    }
}

pub fn server_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ServerChannel::channels_config(),
        receive_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
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
