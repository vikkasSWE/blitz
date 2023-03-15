use bevy::prelude::*;
use bevy_renet::{
    renet::{
        DefaultChannel, RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig,
        ServerEvent,
    },
    RenetServerPlugin,
};
use blitz_common::{panic_on_error_system, setup};

use std::time::SystemTime;
use std::{collections::HashMap, net::UdpSocket};

use serde::{Deserialize, Serialize};

const PROTOCOL_ID: u64 = 7;

const PLAYER_MOVE_SPEED: f32 = 1.0;

#[derive(Debug, Default, Serialize, Deserialize, Component, Resource)]
struct PlayerInput {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[derive(Debug, Component)]
struct Player {
    id: u64,
}

#[derive(Debug, Default, Resource)]
struct Lobby {
    players: HashMap<u64, Entity>,
}

#[derive(Debug, Serialize, Deserialize, Component)]
enum ServerMessages {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
}

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Blitz Server".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }));
    app.init_resource::<Lobby>();

    app.add_plugin(RenetServerPlugin::default());
    app.insert_resource(new_renet_server());

    app.add_systems((
        server_update_system,
        server_sync_players,
        move_players_system,
    ));

    app.add_startup_system(setup);
    app.add_system(panic_on_error_system);

    app.run();
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                // Spawn player cube
                let player_entity = commands
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .id();

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.
                for &player_id in lobby.players.keys() {
                    let message =
                        bincode::serialize(&ServerMessages::PlayerConnected { id: player_id })
                            .unwrap();
                    server.send_message(*id, DefaultChannel::Reliable, message);
                }

                lobby.players.insert(*id, player_entity);

                let message = bincode::serialize(&ServerMessages::PlayerConnected { id: *id })
                    .expect("serialize error");
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message = bincode::serialize(&ServerMessages::PlayerDisconnected { id: *id })
                    .expect("serialize error");
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            let player_input: PlayerInput =
                bincode::deserialize(&message).expect("deserialize error");
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(player_input);
            }
        }
    }
}

fn server_sync_players(mut server: ResMut<RenetServer>, query: Query<(&Transform, &Player)>) {
    let mut players: HashMap<u64, [f32; 3]> = HashMap::new();
    for (transform, player) in query.iter() {
        players.insert(player.id, transform.translation.into());
    }

    let sync_message = bincode::serialize(&players).expect("serialize error");
    server.broadcast_message(DefaultChannel::Unreliable, sync_message);
}

fn move_players_system(mut query: Query<(&mut Transform, &PlayerInput)>, time: Res<Time>) {
    for (mut transform, input) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.down as i8 - input.up as i8) as f32;
        transform.translation.x += x * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();
        transform.translation.z += y * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();
    }
}
