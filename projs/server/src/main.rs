mod collisions;
mod networking;
mod players;
mod projectiles;

use bevy::prelude::*;

use blitz_common::panic_on_error_system;

use crate::{
    collisions::CollisionsPlugin, networking::NetworkPlugin, players::PlayerPlugin,
    projectiles::ProjectilesPlugin,
};

fn main() {
    println!("Starting Blitz Server...");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_plugin(PlayerPlugin);
    app.add_plugin(NetworkPlugin);
    app.add_plugin(ProjectilesPlugin);
    app.add_plugin(CollisionsPlugin);

    app.add_system(panic_on_error_system);

    println!("Blitz Server Running!");
    app.run();
}
