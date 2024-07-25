use crate::render::Dimension;

use project::{RenderSettings, VentApplicationProject};
use render::{camera::camera_controller3d::CameraController3D, DefaultRuntimeRenderer};

use util::{crash::init_panic_hook, input_handler::InputHandler, version::Version};
use vent_logging::Logger;
use vent_window::{Window, WindowAttribs, WindowEvent};

pub mod project;
pub mod render;
pub mod util;

pub struct VentApplication {
    project: VentApplicationProject,
}

impl VentApplication {
    pub fn default() {
        init_panic_hook();
        Logger::init();

        let project = VentApplicationProject {
            name: "Placeholder".to_string(),
            version: Version::new(1, 0, 0),
            window_settings: WindowAttribs::default().with_title("Placeholder".to_string()),
            render_settings: RenderSettings {
                dimension: Dimension::D3,
                vsync: false,
            },
        };
        let app = VentApplication::new(project);
        app.start();
    }

    pub fn new(project: VentApplicationProject) -> Self {
        Self { project }
    }

    pub fn start(self) {
        let project = self.project;
        let app_window = Window::new(project.window_settings.clone());

        // TODO
        let mut renderer = DefaultRuntimeRenderer::new(&project, &app_window);

        let mut input_handler = InputHandler::default();

        let mut controller = CameraController3D::new(5.0, 1.0);
        let mut delta_time = 0.0;

        // TODO, Handle scale factor change
        app_window.poll(move |event| {
            controller.process_keyboard(
                renderer.camera.downcast_mut().expect("TODO"),
                &input_handler,
                delta_time,
            );
            renderer.progress_event(&event);
            match event {
                WindowEvent::Close => {} // Closes automaticly
                WindowEvent::Key { key, state } => input_handler.set_key(key, state),
                WindowEvent::MouseButton { button, state } => {
                    controller.process_mouse_input(&button, &state);
                }
                WindowEvent::Resize {
                    new_width,
                    new_height,
                } => {
                    renderer.resize((new_width, new_height));
                }
                WindowEvent::Draw => delta_time = renderer.render(),
                WindowEvent::MouseMotion { x, y } => controller.process_mouse_movement(
                    renderer.camera.downcast_mut().expect("TODO"),
                    x,
                    y,
                    delta_time,
                ), // Default,
            }
        });
    }
}
