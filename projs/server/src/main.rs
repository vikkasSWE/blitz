use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::collide_aabb::collide,
};
use bevy_renet::{
    renet::{RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use blitz_common::{
    panic_on_error_system, server_connection_config, ClientChannel, FromPlayer, NetworkedEntities,
    Player, PlayerCommand, PlayerInput, Projectile, ServerChannel, ServerMessage,
    PLAYER_MOVE_SPEED, PROTOCOL_ID,
};

use std::{collections::HashMap, f32::consts::FRAC_PI_2, net::UdpSocket};
use std::{f32::consts::PI, time::SystemTime};

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
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

fn main() {
    println!("Starting Blitz Server...");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(RenetServerPlugin::default());

    app.insert_resource(ServerLobby::default());
    app.insert_resource(new_renet_server());

    app.add_systems((
        server_update,
        server_sync_entities,
        move_players,
        move_projectiles,
        update_projectiles,
        projectile_on_removal,
        projectile_hit_player,
    ));

    app.add_system(panic_on_error_system);

    println!("Blitz Server Running!");
    app.run();
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

fn move_players(mut query: Query<(&mut Transform, &PlayerInput)>, time: Res<Time>) {
    for (mut transform, input) in query.iter_mut() {
        let player_pos = vec2(transform.translation.x, transform.translation.y);

        let mut angle = (input.mouse - player_pos).angle_between(Vec2::X) + FRAC_PI_2;

        if angle.is_nan() {
            angle = 0.0;
        }

        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.down as i8 - input.up as i8) as f32;
        transform.translation.x += x * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();
        transform.translation.y -= y * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();
        transform.rotation = Quat::from_rotation_z(-angle);
    }
}

fn move_projectiles(mut query: Query<(&mut Transform, &Projectile)>, time: Res<Time>) {
    for (mut transform, _) in query.iter_mut() {
        let (rotation, mut angle) = transform.rotation.to_axis_angle();

        angle = -angle + FRAC_PI_2;

        if rotation.z.is_sign_positive() {
            angle = -angle + PI;
        }

        transform.translation.x += PLAYER_MOVE_SPEED * time.delta().as_secs_f32() * angle.cos();
        transform.translation.y += PLAYER_MOVE_SPEED * time.delta().as_secs_f32() * angle.sin();
    }
}

fn update_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut projectile) in projectiles.iter_mut() {
        projectile.duration.tick(time.delta());
        if projectile.duration.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_on_removal(
    mut server: ResMut<RenetServer>,
    mut removed_projectiles: RemovedComponents<Projectile>,
) {
    for entity in &mut removed_projectiles {
        let message = ServerMessage::DespawnProjectile { entity };
        let message = bincode::serialize(&message).unwrap();

        server.broadcast_message(ServerChannel::ServerMessages, message);
    }
}

fn projectile_hit_player(
    projectile_query: Query<(Entity, &FromPlayer, &Transform), With<Projectile>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
) {
    for (projectile_entity, from_player, projectile_transform) in projectile_query.iter() {
        for (player_entity, player_tranform) in player_query.iter() {
            if player_entity != from_player.entity
                && collide(
                    player_tranform.translation,
                    vec2(64.0, 64.0),
                    projectile_transform.translation,
                    vec2(64.0, 64.0),
                )
                .is_some()
            {
                println!(
                    "laser {:?} hit player: {:?}",
                    projectile_entity, player_entity
                );
            }
        }
    }
}
