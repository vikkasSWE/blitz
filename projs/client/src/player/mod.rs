use bevy::{math::vec2, prelude::*};
use blitz_common::{PlayerCommand, PlayerInput};

use crate::exit::exit_system;

pub struct ClientPlayerPlugin;
impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>();

        app.add_system(
            player_input
                .run_if(bevy_renet::client_connected)
                .after(exit_system),
        );
    }
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_input: ResMut<PlayerInput>,
    windows: Query<&Window>,
    mut player_commands: EventWriter<PlayerCommand>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    player_input.left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    player_input.right =
        keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    player_input.up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    player_input.down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);

    let window = windows.get_single().unwrap();

    if let Some(mouse) = window.cursor_position() {
        player_input.mouse = mouse - vec2(window.width() / 2.0, window.height() / 2.0);
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        player_commands.send(PlayerCommand::BasicAttack);
    }
}
