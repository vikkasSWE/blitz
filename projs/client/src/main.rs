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

use networking::ClientNetworkPlugin;
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

    app.add_plugin(LdtkPlugin)
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .insert_resource(LevelSelection::Index(0))
        .register_ldtk_entity::<MyBundle>("MyEntityIdentifier");

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

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load(asset_path.join("world.ldtk")),
        ..Default::default()
    });
}

#[derive(Default, Component)]
struct ComponentA;

#[derive(Default, Component)]
struct ComponentB;

#[derive(Bundle, LdtkEntity)]
pub struct MyBundle {
    a: ComponentA,
    b: ComponentB,
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_bundle: SpriteSheetBundle,
}
