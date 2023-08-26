mod audio_capturers;
mod frame_capturers;
mod traits;
mod types;
mod virtual_desktop;

use std::error::Error;

use types::OutputIdentifier;
use virtual_desktop::manager::Manager;

fn main() -> Result<(), Box<dyn Error>> {
    let mut manager = Manager::new()?;
    let status = manager.frame_capturer.capture(
        OutputIdentifier::Name("eDP-1".to_string()),
        types::CaptureType::Stream,
    );
    if let Err(e) = status {
        dbg!(e);
    }
    Ok(())
}
