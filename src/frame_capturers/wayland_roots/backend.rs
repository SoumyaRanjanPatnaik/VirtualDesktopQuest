use crate::traits::FrameCaptureBackend;
use std::error::Error;
use wayland_client::globals::{registry_queue_init, GlobalList};
use wayland_client::protocol::wl_display::WlDisplay;
use wayland_client::{Connection, EventQueue};

use super::state::CapturerState;

pub struct WlrFrameCapturer {
    _state: CapturerState,
    _display: WlDisplay,
    _connection: Connection,
    _event_queue: EventQueue<CapturerState>,
    _global_list: GlobalList,
}

impl WlrFrameCapturer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Create a new wayland connectioni
        let connection = Connection::connect_to_env()?;

        // Event sources
        let (globals, event_queue) = registry_queue_init::<CapturerState>(&connection).unwrap();
        let qh = event_queue.handle();

        // Get wl_display object
        let display = connection.display();

        // Initialize application state
        let state = CapturerState::new(&globals, &qh)?;
        Ok(Self {
            _global_list: globals,
            _connection: connection,
            _event_queue: event_queue,
            _display: display,
            _state: state,
        })
    }
}

impl FrameCaptureBackend for WlrFrameCapturer {
    fn capture(&self, _output_name: String) {
        // self.state.outputs.iter().find(|&output| output.to_owned());
    }
    fn capture_all_outputs(&self) {}
}
