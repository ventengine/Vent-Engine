use vent_common::project::VentApplicationProject;
use vent_runtime::VentApplication;


fn main() {
    env_logger::init();

    let info = VentApplicationProject {
        name: "TODO".to_string(),
        version: "1.0.0".to_string(),
    };
    let app = VentApplication::new(info);
    app.start();
}
