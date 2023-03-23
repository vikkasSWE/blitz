use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use bevy_renet::{
    renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use blitz_common::{
    ClientChannel, FromPlayer, NetworkedEntities, Player, PlayerCommand, PlayerInput, Projectile,
    ServerChannel, ServerMessage, PROTOCOL_ID,
};

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin::default());

        app.insert_resource(ServerLobby::default());
        app.insert_resource(new_renet_server());

        app.add_systems((server_update, server_sync_entities));
    }
}

pub fn server_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ServerChannel::channels_config(),
        receive_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5001".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = server_connection_config();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn server_update(
    mut commands: Commands,
    mut server_events: EventReader<ServerEvent>,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &Player, &Transform)>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Client {id} Connected!!");

                for (entity, player, _) in players.iter() {
                    let message = bincode::serialize(&ServerMessage::PlayerCreate {
                        id: player.id,
                        entity,
                    })
                    .unwrap();
                    server.send_message(*id, ServerChannel::ServerMessages, message);
                }

                let player_entity = commands
                    .spawn(PbrBundle {
                        transform: Transform {
                            translation: vec3(0.0, 0.0, 0.0),
                            scale: vec3(0.5, 0.5, 1.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .id();

                lobby.players.insert(*id, player_entity);

                let message = bincode::serialize(&ServerMessage::PlayerCreate {
                    id: *id,
                    entity: player_entity,
                })
                .expect("Failed to Serialize message!");
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Client {id} Disconnected!!");

                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message = bincode::serialize(&ServerMessage::PlayerDisconnected { id: *id })
                    .expect("Failed to Serialize message!");
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let player_input: PlayerInput =
                bincode::deserialize(&message).expect("Failed to Deserialize message!");

            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(player_input);
            }
        }

        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            let command: PlayerCommand =
                bincode::deserialize(&message).expect("Failed to Deserialize message!");

            match command {
                PlayerCommand::BasicAttack => {
                    println!("Received basic attack from client {}", client_id);

                    if let Some(player_entity) = lobby.players.get(&client_id) {
                        if let Ok((_, _, player_transform)) = players.get(*player_entity) {
                            let projectile_entity = commands
                                .spawn(SpriteBundle {
                                    transform: Transform {
                                        translation: player_transform.translation,
                                        scale: vec3(0.5, 0.5, 1.0),
                                        rotation: player_transform.rotation,
                                    },
                                    ..Default::default()
                                })
                                .insert(Projectile {
                                    duration: Timer::from_seconds(1.5, TimerMode::Once),
                                })
                                .insert(FromPlayer {
                                    entity: *player_entity,
                                })
                                .id();

                            let message = ServerMessage::SpawnProjectile {
                                entity: projectile_entity,
                                transform: vec2(
                                    player_transform.translation.x,
                                    player_transform.translation.y,
                                ),
                                rotation: player_transform.rotation,
                            };
                            let message =
                                bincode::serialize(&message).expect("Failed to Serialize message!");
                            server.broadcast_message(ServerChannel::ServerMessages, message);
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn server_sync_entities(
    mut server: ResMut<RenetServer>,
    query: Query<(Entity, &Transform), Or<(With<Player>, With<Projectile>)>>,
) {
    let mut networked_entities = NetworkedEntities::default();

    for (entity, transform) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities.transforms.push(*transform);
    }

    let sync_message =
        bincode::serialize(&networked_entities).expect("Failed to Serialize message!");
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}
