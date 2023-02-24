use vent_runtime::{AppInfo, VentApplication};

fn main() {
    let info = AppInfo {
        name: "TODO".to_string(),
        version: "1.0.0".to_string(),
    };
    let app = VentApplication::new(info);
    app.start();
}