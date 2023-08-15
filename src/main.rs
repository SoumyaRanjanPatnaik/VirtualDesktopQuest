mod audio_capturers;
mod frame_capturers;
mod traits;
mod types;
mod virtual_desktop;

use std::error::Error;

use virtual_desktop::manager::Manager;

fn main() -> Result<(), Box<dyn Error>> {
    let _manager = Manager::new();
    Ok(())
}
