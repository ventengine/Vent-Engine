use std::path::Path;

pub struct VentApplicationProject<'a> {
    pub name: String,
    pub working_dir: &'a Path,
    pub version: String,
}

impl VentApplicationProject<'_>  {
    /// Creates a New Project
    pub fn serialize() {

    }
}

