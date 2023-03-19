use bevy::prelude::*;
use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    RenetClientPlugin,
};
use blitz_common::{
    panic_on_error_system, Lobby, Player, PlayerInput, ServerMessages, PROTOCOL_ID,
};
use exit::exit_system;

use std::time::SystemTime;
use std::{collections::HashMap, net::UdpSocket};

mod exit;

fn new_renet_client() -> RenetClient {
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

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Blitz Client".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }));

    app.init_resource::<Lobby>();

    app.add_plugin(RenetClientPlugin::default());
    app.insert_resource(new_renet_client());
    app.init_resource::<PlayerInput>();

    app.add_systems(
        (player_input, client_sync_players, client_send_input)
            .distributive_run_if(bevy_renet::client_connected),
    );

    app.add_system(panic_on_error_system);
    app.add_system(exit_system);

    app.run();
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    player_input.right =
        keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    player_input.up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    player_input.down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();

    client.send_message(DefaultChannel::Reliable, input_message);
}

fn client_sync_players(mut lobby: ResMut<Lobby>, mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerConnected { id } => {
                println!("Player {} connected.", id);

                lobby.players.insert(id, Player::default());
            }
            ServerMessages::PlayerDisconnected { id } => {
                println!("Player {} disconnected.", id);
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let players: HashMap<u64, Player> = bincode::deserialize(&message).unwrap();

        for (player_id, new_player) in players.iter() {
            if let Some(player) = lobby.players.get_mut(player_id) {
                if new_player.input != player.input {
                    println!("Player {player_id} moved to {new_player:?}")
                }

                *player = *new_player;
            }
        }
    }
}
