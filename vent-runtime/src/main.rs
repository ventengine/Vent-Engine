use vent_common::window::VentWindow;
use wgpu::SurfaceError;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::lib::{AppInfo, VentApplication};
use vent_common::render::Renderer;

mod lib;
pub mod render;

fn main() {
    let info = AppInfo {
        name: "TODO".to_string(),
        version: "1.0.0".to_string(),
    };
    let app = VentApplication::new(info);
    app.start();
}
