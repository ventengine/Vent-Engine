use vent_common::project::VentApplicationProject;
use vent_runtime::VentApplication;

fn main() {
    let project = VentApplicationProject {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    project
        .deserialize(env!("CARGO_MANIFEST_DIR"))
        .expect("Failed to write Vent Project");
    VentApplication::default();
}
