use crate::traits::FrameCaptureBackend;
use crate::utils::wayland::wrappers::{EventQueue, Registry};
use std::error::Error;
use wayland_client::protocol::__interfaces::WL_SHM_INTERFACE;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::{
    protocol::{__interfaces::WL_OUTPUT_INTERFACE, wl_display::WlDisplay},
    Connection,
};
use wayland_protocols_wlr::screencopy::v1::client::__interfaces::ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

use super::state::CapturerState;

pub struct WlrFrameCapturer {
    state: CapturerState,
    connection: Connection,
    event_queue: EventQueue<CapturerState>,
    display: WlDisplay,
    wlr_screencopy_manager: ZwlrScreencopyManagerV1,
    wl_shm: WlShm,
}

impl WlrFrameCapturer {
    fn new() -> Result<Self, Box<dyn Error>> {
        // Initialize application state
        let mut state = CapturerState::default();

        // Create a new wayland connectioni
        let connection = Connection::connect_to_env()?;

        // Event sources
        let mut event_queue = EventQueue::new(&connection);
        let qh = event_queue.handle();

        // Get wl_display object
        let display = connection.display();

        let registry = event_queue.wait_for(&mut state, || Registry::new(&display, &qh, ()))??;

        let output_globals = state.get_globals_by_interface_name(WL_OUTPUT_INTERFACE.name);
        state.outputs_old = event_queue.wait_for(&mut state, || {
            registry.get_outputs(&output_globals, &qh, ())
        })?;

        let screencopy_manager_global =
            state.get_global_by_interface_name(ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE.name)?;
        let wlr_screencopy_manager = event_queue.wait_for(&mut state, || {
            registry.bind_global(&screencopy_manager_global, &qh, ())
        })?;

        let wl_shm_global = state.get_global_by_interface_name(WL_SHM_INTERFACE.name)?;
        let wl_shm =
            event_queue.wait_for(&mut state, || registry.bind_global(&wl_shm_global, &qh, ()))?;

        Ok(WlrFrameCapturer {
            state,
            display,
            connection,
            event_queue,
            wlr_screencopy_manager,
            wl_shm,
        })
    }
}

impl FrameCaptureBackend for WlrFrameCapturer {
    fn capture(&self, _output_name: String) {
        // self.state.outputs.iter().find(|&output| output.to_owned());
    }
    fn capture_all_outputs(&self) {}
}
