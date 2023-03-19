use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use blitz_common::{panic_on_error_system, Lobby, PlayerInput};
use exit::exit_system;
use networking::{client_send_input, client_sync_players, new_renet_client};
use resources::{Textures, WinSize, PLAYER_SPRITE};

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

    app.init_resource::<Lobby>();
    app.init_resource::<PlayerInput>();

    app.add_plugin(RenetClientPlugin::default());
    app.insert_resource(new_renet_client());

    app.add_startup_system(setup);

    app.add_systems(
        (player_input, client_sync_players, client_send_input)
            .distributive_run_if(bevy_renet::client_connected),
    );

    app.add_system(panic_on_error_system);
    app.add_system(exit_system);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let window = windows.get_single().unwrap();

    // Camera
    commands.spawn(Camera2dBundle::default());

    // Textures
    commands.insert_resource(Textures {
        player: asset_server.load(PLAYER_SPRITE),
    });

    // Create Window Resources
    commands.insert_resource(WinSize {
        width: window.width(),
        height: window.height(),
    });
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    player_input.right =
        keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    player_input.up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    player_input.down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
}
