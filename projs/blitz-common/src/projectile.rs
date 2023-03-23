use bevy::{prelude::Component, time::Timer};

#[derive(Debug, Component, Default)]
pub struct Projectile {
    pub duration: Timer,
}
