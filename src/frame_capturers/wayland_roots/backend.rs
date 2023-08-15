use crate::traits::FrameCaptureBackend;
use crate::types::OutputIdentifier;
use smithay_client_toolkit::output::OutputHandler;
use std::borrow::Borrow;
use std::error::Error;
use wayland_client::globals::{registry_queue_init, GlobalList};
use wayland_client::protocol::wl_display::WlDisplay;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::{Connection, EventQueue};
use wayland_protocols_wlr::screencopy::v1::client::__interfaces::ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

use super::state::CapturerState;

pub struct WlrFrameCapturer {
    state: CapturerState,
    _display: WlDisplay,
    connection: Connection,
    event_queue: EventQueue<CapturerState>,
    global_list: GlobalList,
}

impl WlrFrameCapturer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Create a new wayland connectioni
        let connection = Connection::connect_to_env()?;

        // Event sources
        let (global_list, mut event_queue) =
            registry_queue_init::<CapturerState>(&connection).unwrap();
        let qh = event_queue.handle();

        // Get wl_display object
        let display = connection.display();

        // Initialize application state
        let mut state = CapturerState::new(&global_list, &qh)?;
        event_queue.roundtrip(&mut state)?;
        Ok(Self {
            global_list,
            connection,
            event_queue,
            _display: display,
            state,
        })
    }
}

impl FrameCaptureBackend for WlrFrameCapturer {
    fn capture(&mut self, identifier: OutputIdentifier) -> Result<(), Box<dyn Error>> {
        let output_state = self.state.output_state();
        let match_output = |out: &WlOutput| {
            let Some(info) = output_state.info(out) else {
                return false;
            };
            match identifier.borrow() {
                OutputIdentifier::Name(name) => {
                    let Some(connector) = &info.name else {
                        return false;
                    };
                    name == connector
                }
                OutputIdentifier::Description(desc) => {
                    let Some(output_description) = &info.description else {
                            return false;
                        };
                    output_description == desc
                }
                OutputIdentifier::Metadata { make, model } => {
                    info.make.as_str() == make && info.model.as_str() == model
                }
            }
        };
        let Some(output) = output_state.outputs().find(match_output) else {
            return Err("Could not find the specified output".into());
        };

        let qh = self.event_queue.handle();
        let screencopy_mgr: ZwlrScreencopyManagerV1 =
            self.global_list
                .bind(&qh, 1..=ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE.version, ())?;
        screencopy_mgr.capture_output(1, &output, &qh, output.clone());
        self.event_queue.roundtrip(&mut self.state)?;
        Ok(())
    }
    fn capture_all_outputs(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
