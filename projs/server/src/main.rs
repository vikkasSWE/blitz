mod collisions;
mod networking;
mod players;
mod projectiles;

use bevy::prelude::*;

use blitz_common::panic_on_error_system;

use crate::{
    collisions::ServerCollisionsPlugin, networking::ServerNetworkPlugin,
    players::ServerPlayerPlugin, projectiles::ServerProjectilesPlugin,
};

fn main() {
    println!("Starting Blitz Server...");

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_plugin(ServerPlayerPlugin);
    app.add_plugin(ServerNetworkPlugin);
    app.add_plugin(ServerProjectilesPlugin);
    app.add_plugin(ServerCollisionsPlugin);

    app.add_system(panic_on_error_system);

    println!("Blitz Server Running!");
    app.run();
}
