use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};

#[derive(Serialize, Deserialize)]
// Basic Project Information's
pub struct VentApplicationProject {
    pub name: String,
    pub version: String,
}

impl VentApplicationProject {
    // Deserialize the project data from a .vent file
    pub fn deserialize(&self, path: &str) -> Result<(), std::io::Error> {
        let path_str = format!("{}/project.vent", path);
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path_str)?;

        serde_json::to_writer(file, self)?;

        Ok(())
    }

    // Serialize the project data from a .vent file
    pub fn serialize(path: &str) -> Result<Self, std::io::Error> {
        let path_str = format!("{}/project.vent", path);
        let file = File::open(path_str)?;

        let project = serde_json::from_reader(file)?;

        Ok(project)
    }
}
