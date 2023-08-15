pub trait AudioCaptureBackend {
    fn capture(&self, output_name: String);
    fn capture_all_outputs(&self);
}

pub trait FrameCaptureBackend {
    fn capture(&self, output_name: String);
    fn capture_all_outputs(&self);
}
