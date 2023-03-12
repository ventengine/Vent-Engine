use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
// Basic Project Information's
pub struct VentApplicationProject {
    pub name: String,
    pub version: String,
}

impl VentApplicationProject {
    // Deserialize the project data from a .vent file
    pub fn deserialize(&self) -> Result<(), std::io::Error> {
        let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/project.vent"));
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, self)?;

        Ok(())
    }

    // Serialize the project data from a .vent file
    pub fn serialize() -> Result<Self, std::io::Error> {
        let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/project.vent"));
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let project = serde_json::from_reader(reader)?;

        Ok(project)
    }
}
