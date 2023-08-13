use crate::traits::FrameCaptureBackend;

pub struct WlrFrameCapturer;

impl FrameCaptureBackend for WlrFrameCapturer {
    fn capture(_output_name: String) {}
    fn capture_all_outputs() {}
}
