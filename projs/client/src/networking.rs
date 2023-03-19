use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_renet::renet::{ClientAuthentication, RenetClient};
use blitz_common::{
    client_connection_config, ClientChannel, Lobby, Player, PlayerInput, ServerChannel,
    ServerMessage, PROTOCOL_ID,
};

use std::{collections::HashMap, f32::consts::PI, net::UdpSocket, time::SystemTime};

use crate::{resources::Textures, PlayerCommand};

pub fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5001".parse().unwrap(); // "192.168.0.6:5001".parse().unwrap(); //
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let connection_config = client_connection_config();
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

    client.send_message(ClientChannel::Input, input_message);
}

pub fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}

pub fn client_sync_players(
    mut commands: Commands,
    textures: Res<Textures>,
    mut lobby: ResMut<Lobby>,
    mut client: ResMut<RenetClient>,
    windows: Query<&Window>,
) {
    let window = windows.get_single().unwrap();

    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message =
            bincode::deserialize(&message).expect("Failed to Deserialize message!");
        match server_message {
            ServerMessage::PlayerConnected { id } => {
                println!("Player {} connected.", id);

                let player_entity = commands
                    .spawn(SpriteBundle {
                        texture: textures.player.clone(),
                        transform: Transform {
                            translation: vec3(0.0, 0.0, 10.0),
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
            ServerMessage::PlayerDisconnected { id } => {
                println!("Player {} disconnected.", id);

                if let Some(player_entity) = lobby.players.remove(&id) {
                    if let Some(entity) = player_entity.entity {
                        commands.entity(entity).despawn();
                    }
                }
            }
            ServerMessage::SpawnProjectile {
                entity,
                translation,
            } => {
                println!("SpawnProjectile message! {entity}, {translation:?}");
                commands.spawn(SpriteBundle {
                    texture: textures.player_laser.clone(),
                    transform: Transform {
                        translation: vec3(translation[0], translation[1], 0.0),
                        scale: vec3(0.5, 0.5, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            ServerMessage::DespawnProjectile { entity } => {
                println!("DespawnProjectile message! {entity}");
                //TODO
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let players: HashMap<u64, [f32; 4]> =
            bincode::deserialize(&message).expect("Failed to Deserialize message!");

        for (player_id, translation) in players.iter() {
            if let Some(player) = lobby.players.get(player_id) {
                if let Some(player_entity) = &player.entity {
                    let player_pos = vec2(translation[0], translation[1]);
                    let mouse_pos = vec2(
                        translation[2] - window.width() / 2.0,
                        translation[3] - window.height() / 2.0,
                    );
                    let angle = (player_pos - mouse_pos).angle_between(Vec2::X) + PI;

                    let transform = Transform {
                        translation: vec3(translation[0], translation[1], 10.0),
                        rotation: Quat::from_rotation_z(-angle - PI / 2.0),
                        scale: vec3(0.5, 0.5, 1.0),
                    };

                    commands.entity(*player_entity).insert(transform);
                }
            }
        }
    }
}