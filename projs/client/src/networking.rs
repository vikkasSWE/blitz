use bevy::{math::vec3, prelude::*};
use bevy_renet::renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig};
use blitz_common::{Lobby, Player, PlayerInput, ServerMessages, PROTOCOL_ID};

use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use crate::resources::{Textures, WinSize};

pub fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5001".parse().unwrap(); // "192.168.0.6:5001".parse().unwrap(); //
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let connection_config = RenetConnectionConfig::default();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

pub fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();

    client.send_message(DefaultChannel::Reliable, input_message);
}

pub fn client_sync_players(
    mut commands: Commands,
    textures: Res<Textures>,
    win_size: Res<WinSize>,
    mut lobby: ResMut<Lobby>,
    mut client: ResMut<RenetClient>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerConnected { id } => {
                println!("Player {} connected.", id);

                let bottom = -win_size.height / 2.0;
                let player_entity = commands
                    .spawn(SpriteBundle {
                        texture: textures.player.clone(),
                        transform: Transform {
                            translation: vec3(0.0, bottom + 75.0 / 4.0 + 5.0, 10.0),
                            scale: vec3(0.5, 0.5, 1.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .id();

                lobby.players.insert(
                    id,
                    Player {
                        entity: Some(player_entity),
                        input: PlayerInput::default(),
                        transform: [0.0, 0.0],
                    },
                );
            }
            ServerMessages::PlayerDisconnected { id } => {
                println!("Player {} disconnected.", id);

                if let Some(player_entity) = lobby.players.remove(&id) {
                    if let Some(entity) = player_entity.entity {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let players: HashMap<u64, [f32; 2]> = bincode::deserialize(&message).unwrap();

        for (player_id, translation) in players.iter() {
            if let Some(player) = lobby.players.get(player_id) {
                if let Some(player_entity) = &player.entity {
                    let bottom = -win_size.height / 2.0;
                    let transform = Transform {
                        translation: vec3(
                            0.0 + translation[0],
                            bottom + translation[1] + 75.0 / 4.0 + 5.0,
                            10.0,
                        ),
                        scale: vec3(0.5, 0.5, 1.0),
                        ..Default::default()
                    };

                    commands.entity(*player_entity).insert(transform);
                }
            }
        }
    }
}
