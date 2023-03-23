use bevy::{log, prelude::EventReader};
use bevy_renet::renet::RenetError;

/// If any error is found we just panic
pub fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        match e {
            RenetError::Netcode(e) => log::error!("Netcode Error: {e}"),
            RenetError::Rechannel(e) => log::error!("Rechannel Error: {e}"),
            RenetError::IO(e) => log::error!("IO Error: {e}"),
        }
        panic!();
    }
}
