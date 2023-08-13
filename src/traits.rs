pub trait AudioCaptureBackend {
    fn capture(output_name: String);
    fn capture_all_outputs();
}

pub trait FrameCaptureBackend {
    fn capture(output_name: String);
    fn capture_all_outputs();
}
