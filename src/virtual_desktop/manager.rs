use std::error::Error;

pub struct Manager;

impl Manager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Manager)
    }
}
