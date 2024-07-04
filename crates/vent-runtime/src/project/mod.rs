use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use vent_window::WindowAttribs;

use crate::{render::Dimension, util::version::Version};

/// Basic Project Information's
#[derive(Serialize, Deserialize)]
pub struct VentApplicationProject {
    // Name of the Application
    pub name: String,
    // Version of the Application
    pub version: Version,
    // Inital Window settings, can be changed later
    pub window_settings: WindowAttribs,
    // Inital Render settings, can be changed later
    pub render_settings: RenderSettings,
}

#[derive(Serialize, Deserialize)]
pub struct RenderSettings {
    // Inital vsync setting, can be changed later
    pub dimension: Dimension,
    pub vsync: bool,
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
        log::debug!("Saving {}", path);
        serde_json::to_writer(file, self)?;

        Ok(())
    }

    // Serialize the project data from a .vent file
    pub fn serialize(path: &str) -> Result<Self, std::io::Error> {
        let path_str = format!("{}/project.vent", path);
        log::debug!("Loading {}", path);
        let file = File::open(path_str)?;

        let project = serde_json::from_reader(file)?;

        Ok(project)
    }
}
