use bevy::{math::vec3, prelude::*};
use bevy_renet::renet::{ClientAuthentication, RenetClient};
use blitz_common::{
    client_connection_config, ClientChannel, Lobby, NetworkedEntities, PlayerInput, ServerChannel,
    ServerMessage, PROTOCOL_ID,
};

use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

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

#[derive(Default, Resource)]
pub struct NetworkMapping(HashMap<Entity, Entity>);

pub fn client_sync_players(
    mut commands: Commands,
    textures: Res<Textures>,
    mut lobby: ResMut<Lobby>,
    mut client: ResMut<RenetClient>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message =
            bincode::deserialize(&message).expect("Failed to Deserialize message!");
        match server_message {
            ServerMessage::PlayerConnected { id, entity } => {
                println!("Player {} connected.", id);

                let player_entity = commands
                    .spawn(SpriteBundle {
                        texture: textures.player.clone(),
                        transform: Transform {
                            translation: vec3(0.0, 0.0, 0.0),
                            scale: vec3(0.5, 0.5, 1.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .id();

                lobby.players.insert(id, player_entity);
                network_mapping.0.insert(entity, player_entity);
            }
            ServerMessage::PlayerDisconnected { id } => {
                println!("Player {} disconnected.", id);

                if let Some(player_entity) = lobby.players.remove(&id) {
                    commands.entity(player_entity).despawn();
                    network_mapping.0.remove(&player_entity);
                }
            }
            ServerMessage::SpawnProjectile {
                entity,
                transform,
                rotation,
            } => {
                println!("SpawnProjectile message! {entity:?}");
                let projectile_entity = commands.spawn(SpriteBundle {
                    texture: textures.player_laser.clone(),
                    transform: Transform {
                        translation: vec3(transform.x, transform.y, 0.0),
                        rotation,
                        scale: Vec3::splat(1.0),
                    },
                    ..Default::default()
                });
                network_mapping.0.insert(entity, projectile_entity.id());
            }
            ServerMessage::DespawnProjectile { entity } => {
                println!("DespawnProjectile message! {entity:?}");
                if let Some(entity) = network_mapping.0.remove(&entity) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    // while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
    //     let players: HashMap<u64, Transform> =
    //         bincode::deserialize(&message).expect("Failed to Deserialize message!");
    //     for (player_id, transform) in players.iter() {
    //         if let Some(player_entity) = lobby.players.get(player_id) {
    //             let transform = *transform;
    //             commands.entity(*player_entity).insert(transform);
    //         }
    //     }
    // }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let transform = networked_entities.transforms[i];

                commands.entity(*entity).insert(transform);
            }
        }
    }
}
