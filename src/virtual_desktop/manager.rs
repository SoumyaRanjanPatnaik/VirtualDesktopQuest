use std::{env, error::Error};

use crate::{
    frame_capturers::wayland_roots::backend::WlrFrameCapturer, traits::FrameCaptureBackend,
};

const DISPLAY_BACKENDS: [&str; 1] = ["wayland_roots"];

pub struct Manager {
    frame_capturer: Box<dyn FrameCaptureBackend>,
}

impl Manager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let session_type = env::var("XDG_SESSION_TYPE")?;
        let frame_capturer: Box<dyn FrameCaptureBackend> = match session_type.as_str() {
            "wayland" => {
                let wlr_backend =
                    WlrFrameCapturer::new().expect("Only wlr based compositors are supported");
                Box::new(wlr_backend)
            }
            _ => {
                panic!("Unsupported session type type - {session_type}. Supported display backends are - {}", DISPLAY_BACKENDS.join(", "))
            }
        };
        Ok(Manager { frame_capturer })
    }
}
