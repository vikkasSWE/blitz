use std::path::Path;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::common_conditions::input_toggle_active,
    prelude::*,
};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use blitz_common::{panic_on_error_system, PlayerCommand};
use exit::exit_system;

use networking::{resources::ControlledPlayer, ClientNetworkPlugin};
use player::ClientPlayerPlugin;
use resources::{Textures, ASSETS_DIR, PLAYER_LASER_SPRITE, PLAYER_SPRITE};

mod exit;
mod networking;
mod player;
mod resources;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Blitz Client".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    app.add_plugin(ClientPlayerPlugin);
    app.add_plugin(ClientNetworkPlugin);
    app.add_plugin(WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::I)));

    app.add_plugin(LdtkPlugin)
        .insert_resource(LevelSelection::Index(0));

    app.add_startup_system(setup);

    app.add_system(move_camera);

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

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load(asset_path.join("world.ldtk")),
        ..Default::default()
    });
}

fn move_camera(
    mut query_camera: Query<&mut Transform, With<Camera>>,
    query_player: Query<&Transform, (With<ControlledPlayer>, Without<Camera>)>,
    time: Res<Time>,
) {
    if let Some(player_transform) = query_player.iter().next() {
        if let Ok(mut camera_transform) = query_camera.get_single_mut() {
            let interpolation_speed = 5.0;

            let old_z = camera_transform.translation.z;

            camera_transform.translation = camera_transform.translation.lerp(
                player_transform.translation,
                time.delta_seconds() * interpolation_speed,
            );

            camera_transform.translation.z = old_z;
        }
    }
}
