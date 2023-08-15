use std::error::Error;

use crate::traits::AudioCaptureBackend;

pub struct PipewireAudioCapturer;

impl AudioCaptureBackend for PipewireAudioCapturer {
    fn capture(&mut self, _output_name: String) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn capture_all_outputs(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
