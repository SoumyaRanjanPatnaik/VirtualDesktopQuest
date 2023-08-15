use std::error::Error;

use crate::types::OutputIdentifier;

pub trait AudioCaptureBackend {
    fn capture(&mut self, identifier: String) -> Result<(), Box<dyn Error>>;
    fn capture_all_outputs(&mut self) -> Result<(), Box<dyn Error>>;
}

pub trait FrameCaptureBackend {
    fn capture(&mut self, identifier: OutputIdentifier) -> Result<(), Box<dyn Error>>;
    fn capture_all_outputs(&mut self) -> Result<(), Box<dyn Error>>;
}
