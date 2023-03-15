use bevy::prelude::*;
use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    RenetClientPlugin,
};
use blitz_common::{panic_on_error_system, PlayerInput, PROTOCOL_ID};

use std::net::UdpSocket;
use std::time::SystemTime;

fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5001".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
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

    app.add_plugin(RenetClientPlugin::default());
    app.insert_resource(new_renet_client());
    app.init_resource::<PlayerInput>();

    app.add_systems(
        (player_input, client_send_input).distributive_run_if(bevy_renet::client_connected),
    );

    app.add_system(panic_on_error_system);

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
