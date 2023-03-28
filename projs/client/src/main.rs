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
use resources::{
    AudioAtlas, Explosion, ExplosionTimer, ExplosionToSpawn, Textures, ASSETS_DIR,
    PLAYER_LASER_SPRITE, PLAYER_SPRITE,
};

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
    app.add_system(explosion_to_spawn);
    app.add_system(animate_explosion);

    app.add_system(panic_on_error_system);
    app.add_system(exit_system);

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
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
    let texture_handle = asset_server.load(asset_path.join("explo_a_sheet.png"));
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2 { x: 64.0, y: 64.0 }, 4, 4, None, None);

    commands.insert_resource(Textures {
        player: asset_server.load(asset_path.join(PLAYER_SPRITE)),
        player_laser: asset_server.load(asset_path.join(PLAYER_LASER_SPRITE)),
        explosion: texture_atlases.add(texture_atlas),
    });

    // Sound
    commands.insert_resource(AudioAtlas {
        player_laser: asset_server.load(asset_path.join("player_laser.ogg")),
    });

    // World
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

fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity, &ExplosionToSpawn)>,
    textures: Res<Textures>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: textures.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(ExplosionTimer::default());

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1;
            if sprite.index >= 16 {
                commands.entity(entity).despawn();
            }
        }
    }
}
