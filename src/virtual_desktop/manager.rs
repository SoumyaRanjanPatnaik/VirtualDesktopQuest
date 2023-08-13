use std::error::Error;

use wayland_client::{
    protocol::{__interfaces::WL_OUTPUT_INTERFACE, wl_display::WlDisplay},
    Connection,
};

use crate::utils::wayland::wrappers::{EventQueue, Registry};
use crate::virtual_desktop::app_data::AppData;

#[allow(dead_code)]
pub struct Manager {
    state: AppData,
    connection: Connection,
    event_queue: EventQueue<AppData>,
    display: WlDisplay,
}

impl Manager {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Initialize application state
        let mut state = AppData::default();

        // Create a new wayland connectioni
        let connection = Connection::connect_to_env()?;

        // Event sources
        let mut event_queue = EventQueue::new(&connection);
        let qh = event_queue.handle();

        // Get wl_display object
        let display = connection.display();

        let registry = event_queue.wait_for(&mut state, || Registry::new(&display, &qh, ()))??;

        let output_globals = state.get_globals_by_interface_name(WL_OUTPUT_INTERFACE.name);
        state.outputs = event_queue.wait_for(&mut state, || {
            registry.get_outputs(&output_globals, &qh, ())
        })?;

        Ok(Manager {
            state,
            display,
            connection,
            event_queue,
        })
    }
}
