use bevy::prelude::{Handle, Image, Resource};

pub const PLAYER_SPRITE: &str = "player_b_01.png";
pub const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";

#[derive(Resource)]
pub struct Textures {
    pub player: Handle<Image>,
    pub player_laser: Handle<Image>,
}

#[derive(Resource)]
pub struct WinSize {
    #[allow(unused)]
    pub width: f32,
    pub height: f32,
}
