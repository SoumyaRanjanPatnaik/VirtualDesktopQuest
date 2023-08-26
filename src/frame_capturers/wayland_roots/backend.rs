use crate::traits::FrameCaptureBackend;
use crate::types::{CaptureType, OutputIdentifier};
use smithay_client_toolkit::output::OutputHandler;
use std::borrow::Borrow;
use std::error::Error;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::{Connection, EventQueue};
use wayland_protocols_wlr::screencopy::v1::client::__interfaces::ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE;
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

use super::state::CapturerState;

pub struct WlrFrameCapturer {
    state: CapturerState,
    event_queue: EventQueue<CapturerState>,
}

impl WlrFrameCapturer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Create a new wayland connectioni
        let connection = Connection::connect_to_env()?;

        // Event sources
        let (global_list, mut event_queue) =
            registry_queue_init::<CapturerState>(&connection).unwrap();
        let qh = event_queue.handle();

        // Initialize application state
        let mut state = CapturerState::new(global_list, &qh)?;
        event_queue.roundtrip(&mut state)?;
        Ok(Self { event_queue, state })
    }
}

impl FrameCaptureBackend for WlrFrameCapturer {
    fn capture(
        &mut self,
        identifier: OutputIdentifier,
        r#type: CaptureType,
    ) -> Result<(), Box<dyn Error>> {
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
        self.state.poll_capture(&qh, r#type.clone(), &output)?;
        if let CaptureType::Stream = &r#type {
            loop {
                println!("EVENT RECIEVED...");
                self.event_queue.blocking_dispatch(&mut self.state).unwrap();
            }
        };
        Ok(())
    }
    fn capture_all_outputs(&mut self, _type: CaptureType) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn stop_capture(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
