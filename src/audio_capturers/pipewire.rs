use crate::traits::AudioCaptureBackend;

pub struct PipewireAudioCapturer;

impl AudioCaptureBackend for PipewireAudioCapturer {
    fn capture(_output_name: String) {}
    fn capture_all_outputs() {}
}
