use bevy::{
    prelude::{AudioSource, Component, Handle, Image, Resource, Vec3},
    sprite::TextureAtlas,
    time::{Timer, TimerMode},
};

pub static ASSETS_DIR: &str = env!("ASSETS_DIR");

pub const PLAYER_SPRITE: &str = "player_b_01.png";
pub const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";

#[derive(Resource)]
pub struct Textures {
    pub player: Handle<Image>,
    pub player_laser: Handle<Image>,
    pub explosion: Handle<TextureAtlas>,
}

#[derive(Resource)]
pub struct WinSize {
    #[allow(unused)]
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct AudioAtlas {
    pub player_laser: Handle<AudioSource>,
}

#[derive(Component)]
pub struct Explosion;
#[derive(Component)]
pub struct ExplosionToSpawn(pub Vec3);
#[derive(Component)]
pub struct ExplosionTimer(pub Timer);

impl Default for ExplosionTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.05, TimerMode::Repeating))
    }
}
