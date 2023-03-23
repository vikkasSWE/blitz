use std::f32::consts::FRAC_PI_2;

use bevy::{math::vec2, prelude::*};
use blitz_common::{PlayerInput, PLAYER_MOVE_SPEED};

pub struct ServerPlayerPlugin;
impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_players);
    }
}

fn move_players(mut query: Query<(&mut Transform, &PlayerInput)>, time: Res<Time>) {
    for (mut transform, input) in query.iter_mut() {
        let player_pos = vec2(transform.translation.x, transform.translation.y);

        let mut angle = (input.mouse - player_pos).angle_between(Vec2::X) + FRAC_PI_2;

        if angle.is_nan() {
            angle = 0.0;
        }

        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.down as i8 - input.up as i8) as f32;
        transform.translation.x += x * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();
        transform.translation.y -= y * PLAYER_MOVE_SPEED * time.delta().as_secs_f32();
        transform.rotation = Quat::from_rotation_z(-angle);
    }
}
