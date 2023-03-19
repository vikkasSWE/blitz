use bevy::{
    log,
    prelude::{EventReader, Input, KeyCode, Res, ResMut},
    window::WindowCloseRequested,
};
use bevy_renet::renet::RenetClient;

pub fn exit_system(
    events: EventReader<WindowCloseRequested>,
    mut client: ResMut<RenetClient>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if !events.is_empty() || keyboard_input.pressed(KeyCode::Escape) {
        log::info!("Disconnecting from Server...");
        client.disconnect();
    }
}
