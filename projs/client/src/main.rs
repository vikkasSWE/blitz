use std::path::Path;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
};
use bincode::de;
use blitz_common::{panic_on_error_system, PlayerCommand};
use exit::exit_system;

use networking::ClientNetworkPlugin;
use player::ClientPlayerPlugin;
use resources::{Textures, ASSETS_DIR, PLAYER_LASER_SPRITE, PLAYER_SPRITE};

mod exit;
mod networking;
mod player;
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

    app.add_plugin(ClientPlayerPlugin);
    app.add_plugin(ClientNetworkPlugin);

    app.add_startup_system(setup);

    app.add_system(panic_on_error_system);
    app.add_system(exit_system);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            tonemapping: Tonemapping::Reinhard,
            ..Default::default()
        },
        BloomSettings::NATURAL,
    ));

    let asset_path = Path::new(ASSETS_DIR);

    // Textures
    commands.insert_resource(Textures {
        player: asset_server.load(asset_path.join(PLAYER_SPRITE)),
        player_laser: asset_server.load(asset_path.join(PLAYER_LASER_SPRITE)),
    });
}
