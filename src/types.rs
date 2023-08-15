#[derive(Debug, Clone)]
pub enum OutputIdentifier {
    Name(String),
    Description(String),
    Metadata { make: String, model: String },
}
