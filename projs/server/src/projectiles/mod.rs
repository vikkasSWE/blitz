use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use blitz_common::{Projectile, ServerChannel, ServerMessage, PLAYER_MOVE_SPEED};

pub struct ServerProjectilesPlugin;
impl Plugin for ServerProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((move_projectiles, update_projectiles, projectile_on_removal));
    }
}

fn move_projectiles(mut query: Query<(&mut Transform, &Projectile)>, time: Res<Time>) {
    for (mut transform, _) in query.iter_mut() {
        let (rotation, mut angle) = transform.rotation.to_axis_angle();

        angle = -angle + FRAC_PI_2;

        if rotation.z.is_sign_positive() {
            angle = -angle + PI;
        }

        transform.translation.x += PLAYER_MOVE_SPEED * time.delta().as_secs_f32() * angle.cos();
        transform.translation.y += PLAYER_MOVE_SPEED * time.delta().as_secs_f32() * angle.sin();
    }
}

fn update_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut projectile) in projectiles.iter_mut() {
        projectile.duration.tick(time.delta());
        if projectile.duration.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_on_removal(
    mut server: ResMut<RenetServer>,
    mut removed_projectiles: RemovedComponents<Projectile>,
) {
    for entity in &mut removed_projectiles {
        let message = ServerMessage::DespawnProjectile { entity };
        let message = bincode::serialize(&message).unwrap();

        server.broadcast_message(ServerChannel::ServerMessages, message);
    }
}
