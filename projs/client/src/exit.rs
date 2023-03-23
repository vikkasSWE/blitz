use bevy::{
    app::AppExit,
    log,
    prelude::{EventReader, EventWriter, Input, KeyCode, Res, ResMut},
    window::WindowCloseRequested,
};
use bevy_renet::renet::RenetClient;

pub fn exit_system(
    events: EventReader<WindowCloseRequested>,
    mut exit: EventWriter<AppExit>,
    mut client: Option<ResMut<RenetClient>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Some(client) = &mut client {
        if !events.is_empty() || keyboard_input.pressed(KeyCode::Escape) {
            log::info!("Disconnecting from Server...");
            client.disconnect();

            log::info!("Exiting Application...");
            exit.send(AppExit);
        }

        // TODO: Should be relplaced by a GUI lobby
        // if keyboard_input.pressed(KeyCode::Q) {
        //     log::info!("Disconnecting from Server...");
        //     client.disconnect();
        // }

        // if keyboard_input.pressed(KeyCode::E) {
        //     log::info!("Connecting from Server...");
        //     client();
        // }
    }
}
