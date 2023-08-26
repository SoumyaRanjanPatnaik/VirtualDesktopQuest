use std::error::{self, Error};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum OutputIdentifier {
    Name(String),
    Description(String),
    Metadata { make: String, model: String },
}

#[derive(thiserror::Error, Debug)]
pub enum CapturerError {
    #[error("Invalid transition to {to}: \n\texpected one of: {:?}\n\twas: {found}", expected.join(", "))]
    InvalidTransition {
        found: String,
        expected: Vec<String>,
        to: String,
    },
}

#[derive(Debug, Clone)]
pub enum CapturerStatus {
    NotStarted,
    Pending {
        r#type: CaptureType,
    },
    Running {
        r#type: CaptureType,
        last_frame_at: Option<f64>,
    },
    Done,
    Failed,
}

impl CapturerStatus {
    pub fn get_type(&self) -> String {
        match self {
            Self::NotStarted => String::from("NotStarted"),
            Self::Pending { .. } => String::from("Pending"),
            Self::Running { .. } => String::from("Running"),
            Self::Failed => String::from("Failed( {error} )"),
            Self::Done => String::from("Done"),
        }
    }
    pub fn reset(&mut self) -> Result<(), CapturerError> {
        match self {
            Self::NotStarted | Self::Done | Self::Failed { .. } => {
                *self = Self::NotStarted;
                Ok(())
            }
            Self::Pending { .. } | Self::Running { .. } => Err(CapturerError::InvalidTransition {
                found: self.get_type(),
                expected: vec![
                    "NotStarted".to_string(),
                    "Done".to_string(),
                    "Failed".to_string(),
                ],
                to: Self::NotStarted.get_type(),
            }),
        }
    }
    pub fn initialize(&mut self, r#type: CaptureType) -> Result<(), CapturerError> {
        let initialized_status = Self::Pending { r#type };
        match self {
            Self::NotStarted | Self::Done | Self::Failed { .. } => {
                *self = initialized_status;
                return Ok(());
            }
            Self::Pending { .. } | Self::Running { .. } => Err(CapturerError::InvalidTransition {
                found: self.get_type(),
                expected: vec![
                    "NotStarted".to_string(),
                    "Done".to_string(),
                    "Failed".to_string(),
                ],
                to: initialized_status.get_type(),
            }),
        }
    }
    pub fn start(&mut self) -> Result<(), CapturerError> {
        match self {
            Self::NotStarted | Self::Done | Self::Failed { .. } | Self::Running { .. } => {
                return Err(CapturerError::InvalidTransition {
                    found: self.get_type(),
                    expected: vec!["Pending".to_string()],
                    to: "Running".to_string(),
                })
            }
            Self::Pending { r#type } => {
                *self = Self::Running {
                    r#type: r#type.clone(),
                    last_frame_at: None,
                };
                return Ok(());
            }
        }
    }

    pub fn done(&mut self) -> Result<(), CapturerError> {
        match self {
            Self::NotStarted | Self::Done | Self::Failed { .. } | Self::Running { .. } => {
                return Err(CapturerError::InvalidTransition {
                    found: self.get_type(),
                    expected: vec!["Running".to_string()],
                    to: "Started".to_string(),
                })
            }
            Self::Pending { .. } => {
                *self = Self::Done;
                return Ok(());
            }
        }
    }
    pub fn errored(&mut self, error: Box<dyn Error>) -> Result<(), CapturerError> {
        match self {
            Self::NotStarted | Self::Done | Self::Pending { .. } | Self::Failed { .. } => {
                return Err(CapturerError::InvalidTransition {
                    found: self.get_type(),
                    expected: vec!["Running".to_string()],
                    to: "Started".to_string(),
                })
            }
            Self::Running { .. } => {
                *self = Self::Failed;
                return Ok(());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CaptureType {
    Frame,
    Stream,
}
