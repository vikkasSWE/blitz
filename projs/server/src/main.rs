use bevy::prelude::*;
use bevy_renet::{
    renet::{
        DefaultChannel, RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig,
        ServerEvent,
    },
    RenetServerPlugin,
};
use blitz_common::{
    panic_on_error_system, Lobby, Player, PlayerInput, ServerMessages, PROTOCOL_ID,
};

use std::time::SystemTime;
use std::{collections::HashMap, net::UdpSocket};

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5001".parse().unwrap();
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
    println!("Starting Blitz Server...");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<Lobby>();

    app.add_plugin(RenetServerPlugin::default());
    app.insert_resource(new_renet_server());

    app.add_systems((server_update_system, server_sync_players));

    app.add_system(panic_on_error_system);

    println!("Blitz Server Running!");
    app.run();
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Client {id} Connected!!");

                lobby.players.insert(
                    *id,
                    Player {
                        input: PlayerInput::default(),
                    },
                );

                let message =
                    bincode::serialize(&ServerMessages::PlayerConnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Client {id} Disconnected!!");

                let message =
                    bincode::serialize(&ServerMessages::PlayerDisconnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            let player_input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_data) = lobby.players.get_mut(&client_id) {
                if player_data.input != player_input {
                    println!("Client {client_id} input: {:?}", player_input);
                }

                player_data.input = player_input;
            }
        }
    }
}

fn server_sync_players(mut server: ResMut<RenetServer>, lobby: Res<Lobby>) {
    let mut players: HashMap<u64, Player> = HashMap::new();
    for (id, player) in lobby.players.iter() {
        players.insert(*id, *player);
    }

    let sync_message = bincode::serialize(&players).unwrap();
    server.broadcast_message(DefaultChannel::Unreliable, sync_message);
}
