use vent_common::project::VentApplicationProject;
use vent_runtime::VentApplication;

fn main() {
    let project = VentApplicationProject {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    project.deserialize().expect("Failed to load Vent Project");
    VentApplication::default();
}
