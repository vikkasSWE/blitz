use bevy::{math::vec2, prelude::*, sprite::collide_aabb::collide};
use bevy_renet::renet::RenetServer;
use blitz_common::{FromPlayer, Player, Projectile, ServerChannel, ServerMessage};

pub struct ServerCollisionsPlugin;
impl Plugin for ServerCollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(projectile_hit_player);
    }
}

fn projectile_hit_player(
    mut commands: Commands,
    projectile_query: Query<(Entity, &FromPlayer, &Transform), With<Projectile>>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut server: ResMut<RenetServer>,
) {
    for (projectile_entity, from_player, projectile_transform) in projectile_query.iter() {
        for (player_entity, player_tranform) in player_query.iter() {
            if player_entity != from_player.entity
                && collide(
                    player_tranform.translation,
                    vec2(32.0, 32.0),
                    projectile_transform.translation,
                    vec2(32.0, 32.0),
                )
                .is_some()
            {
                commands.entity(projectile_entity).despawn();
                // commands.entity(player_entity).despawn();

                let messages = vec![
                    bincode::serialize(&ServerMessage::DespawnProjectile {
                        entity: projectile_entity,
                    })
                    .unwrap(),
                    bincode::serialize(&ServerMessage::DespawnPlayer {
                        entity: player_entity,
                    })
                    .unwrap(),
                ];

                for message in messages {
                    server.broadcast_message(ServerChannel::ServerMessages, message);
                }
            }
        }
    }
}
