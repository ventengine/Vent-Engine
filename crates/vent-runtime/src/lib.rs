use crate::render::Dimension;

use render::camera::camera_controller3d::CameraController3D;
use render::DefaultRuntimeRenderer;
use simple_logger::SimpleLogger;
use vent_common::project::VentApplicationProject;

use vent_common::util::crash::init_panic_hook;
use vent_common::window::VentWindow;
use winit::{event::{DeviceEvent, Event, WindowEvent}, window::WindowAttributes};

pub mod render;

pub struct VentApplication {
    project: VentApplicationProject,
}

impl VentApplication {
    pub fn default() {
        init_panic_hook();
        #[cfg(not(target_arch = "wasm32"))]
        {
            SimpleLogger::new()
                .with_level(log::LevelFilter::Info)
                .init()
                .unwrap();
        };

        let project = VentApplicationProject {
            name: "Placeholder".to_string(),
            version: "1.0.0".to_string(),
        };
        let app = VentApplication::new(project);
        app.start();
    }

    pub fn new(project: VentApplicationProject) -> Self {
        Self { project }
    }

    pub fn start(self) {
        let window_builder = WindowAttributes::default().with_title(self.project.name);
        let vent_window = VentWindow::new(window_builder);

        // TODO
        let mut renderer = DefaultRuntimeRenderer::new(
            Dimension::D3,
            &vent_window.window,
        );

        let mut controller = CameraController3D::new(1000.0, 10.0);
        let mut delta_time = 0.0;

        vent_window
            .event_loop
            .run(move |event, elwt| {
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id: _,
                    } => {
                        renderer.progress_event(event);

                        match event {
                            WindowEvent::CloseRequested => elwt.exit(),
                            WindowEvent::MouseInput { button, state, .. } => {
                                controller.process_mouse_input(&vent_window.window, button, state);
                            }
                            WindowEvent::KeyboardInput { event, .. } => {
                                controller.process_keyboard(
                                    renderer.camera.downcast_mut().expect("TODO"),
                                    event,
                                    delta_time,
                                );
                            }
                            WindowEvent::Resized(physical_size) => {
                                renderer.resize(physical_size);
                            }
                            // WindowEvent::ScaleFactorChanged {
                            //     inner_size_writer, ..
                            // } => {
                            //     // new_inner_size is &mut so w have to dereference it twice
                            //     renderer.resize(new_inner_size, &mut cam);
                            // }
                            WindowEvent::RedrawRequested => {
                                delta_time = renderer.render(&vent_window.window);
                            }
                            _ => {}
                        }
                    }
                    Event::AboutToWait {} => vent_window.window.request_redraw(),
                    Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { delta },
                        ..
                    } => controller.process_mouse_movement(
                        renderer.camera.downcast_mut().expect("TODO"),
                        delta.0,
                        delta.1,
                        delta_time,
                    ),

                    // ...
                    _ => {}
                }
            })
            .expect("Window Event Loop Error");
    }
}
