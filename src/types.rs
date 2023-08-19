#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum OutputIdentifier {
    Name(String),
    Description(String),
    Metadata { make: String, model: String },
}
