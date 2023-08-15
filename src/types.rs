#[derive(Debug, Clone, Default)]
pub struct OutputMetadata {
    pub name: String,
    pub description: Option<String>,
    pub mode: (i32, i32, i32),
    pub scale: i32,
}
