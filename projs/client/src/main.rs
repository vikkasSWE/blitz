use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use blitz_common::{panic_on_error_system, Lobby, PlayerCommand, PlayerInput};
use exit::exit_system;
use networking::{
    client_send_input, client_send_player_commands, client_sync_players, new_renet_client,
};
use resources::{Textures, PLAYER_LASER_SPRITE, PLAYER_SPRITE};

mod exit;
mod networking;
mod resources;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Blitz Client".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }));

    app.add_event::<PlayerCommand>();

    app.init_resource::<Lobby>();
    app.init_resource::<PlayerInput>();

    app.add_plugin(RenetClientPlugin::default());
    app.insert_resource(new_renet_client());

    app.add_startup_system(setup);

    app.add_systems(
        (
            player_input.run_if(bevy_renet::client_connected),
            client_sync_players.run_if(bevy_renet::client_connected),
            client_send_input.run_if(bevy_renet::client_connected),
            client_send_player_commands.run_if(bevy_renet::client_connected),
        )
            .after(exit_system),
    );

    app.add_system(panic_on_error_system);
    app.add_system(exit_system);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Textures
    commands.insert_resource(Textures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
    });
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_input: ResMut<PlayerInput>,
    windows: Query<&Window>,
    mut player_commands: EventWriter<PlayerCommand>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    player_input.left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    player_input.right =
        keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    player_input.up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    player_input.down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);

    let window = windows.get_single().unwrap();

    if let Some(mouse) = window.cursor_position() {
        player_input.mouse = mouse;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        player_commands.send(PlayerCommand::BasicAttack {
            cast_at: player_input.mouse,
        });
    }
}
