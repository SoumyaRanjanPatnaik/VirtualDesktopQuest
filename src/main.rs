mod audio_capturers;
mod frame_capturers;
mod traits;
mod utils;
mod virtual_desktop;
mod types;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // let shm_factory = ShmFactoryBuilder::default()
    //     .name("/wl_scrcpy_vd")
    //     .size(1920 * 1080)
    //     .rw(true)
    //     .create(true)
    //     .build()?;
    // let shm = shm_factory.create_shm_map().ok_or("Failed to create shm")?;
    // let _shm_pool = unsafe { shm_mgr.create_pool(shm.fd(), shm.len().try_into()?, &qh, ()) };
    // shm_pool.create_buffer(offset, width, height, stride, format, &qh, udata);

    // for output in &outputs {
    //     let _frame = with_roundtrip(&mut event_queue, &mut app_state, || {
    //         screencopy_mgr.capture_output(1, output, &qh, ())
    //     })?;
    //     // frame.copy_with_damage()
    // }

    // let manager = Manaer::new();
    // manager.start_capture();

    loop {
        // event_queue.blocking_dispatch(&mut app_state)?;
    }
}
