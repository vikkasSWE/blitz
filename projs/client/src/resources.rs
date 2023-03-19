use bevy::prelude::{Handle, Image, Resource};

pub const PLAYER_SPRITE: &str = "../../../assets/player_b_01.png";

#[derive(Resource)]
pub struct Textures {
    pub player: Handle<Image>,
}

#[derive(Resource)]
pub struct WinSize {
    #[allow(unused)]
    pub width: f32,
    pub height: f32,
}
