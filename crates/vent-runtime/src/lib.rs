use std::process::exit;

use crate::render::Dimension;

use project::{RenderSettings, VentApplicationProject};
use render::{camera::camera_controller3d::CameraController3D, DefaultRuntimeRenderer};

use util::{crash::init_panic_hook, version::Version};
use vent_logging::Logger;
use vent_window::{EventLoop, Window, WindowAttribs, WindowEvent};

pub mod project;
pub mod render;
pub mod util;

pub struct VentApplication {
    project: VentApplicationProject,
}

impl VentApplication {
    pub fn default() {
        init_panic_hook();
        Logger::new();

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
        let mut event_loop = EventLoop::new();
        let app_window = Window::new(project.window_settings.clone());

        // TODO
        let mut renderer = DefaultRuntimeRenderer::new(&project, &app_window);

        let mut controller = CameraController3D::new(1000.0, 10.0);
        let mut delta_time = 0.0;

        event_loop.add_window(app_window);

        event_loop.poll(move |event| {
            match event {
                WindowEvent::Close => exit(0), // maybe not so pretty,
                WindowEvent::Key { key, state } => {
                    controller.process_keyboard(
                        renderer.camera.downcast_mut().expect("TODO"),
                        key,
                        state,
                        delta_time,
                    );
                }
                WindowEvent::Mouse { button, state } => {
                    controller.process_mouse_input(&button, &state);
                }
                WindowEvent::Resize {
                    new_width,
                    new_height,
                } => {
                    renderer.resize((new_width, new_height));
                }
                WindowEvent::Draw => delta_time = renderer.render(), // Default,
            }
        });

        //     match event {
        //         Event::WindowEvent {
        //             ref event,
        //             window_id: _,
        //         } => {
        //             renderer.progress_event(event);

        //             match event {
        //                 WindowEvent::CloseRequested => elwt.exit(),
        //                 WindowEvent::MouseInput { button, state, .. } => {
        //                     controller.process_mouse_input(&vent_window.window, button, state);
        //                 }
        //                 WindowEvent::KeyboardInput { event, .. } => {
        // controller.process_keyboard(
        //     renderer.camera.downcast_mut().expect("TODO"),
        //     event,
        //     delta_time,
        //                     );
        //                 }
        //                 WindowEvent::Resized(physical_size) => {
        //                     renderer.resize(physical_size);
        //                 }
        //                 // WindowEvent::ScaleFactorChanged {
        //                 //     inner_size_writer, ..
        //                 // } => {
        //                 //     // new_inner_size is &mut so w have to dereference it twice
        //                 //     renderer.resize(new_inner_size, &mut cam);
        //                 // }
        //                 WindowEvent::RedrawRequested => {
        //                     delta_time = renderer.render(&vent_window.window);
        //                 }
        //                 _ => {}
        //             }
        //         }
        //         Event::AboutToWait {} => vent_window.window.request_redraw(),
        //         Event::DeviceEvent {
        //             event: DeviceEvent::MouseMotion { delta },
        //             ..
        //         } => controller.process_mouse_movement(
        //             renderer.camera.downcast_mut().expect("TODO"),
        //             delta.0,
        //             delta.1,
        //             delta_time,
        //         ),

        //         // ...
        //         _ => {}
        //     }
        // })
        // .expect("Window Event Loop Error");
    }
}
