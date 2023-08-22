use crate::render::{Dimension, RuntimeRenderer};

use render::camera::camera_controller3d::CameraController3D;
use render::camera::{Camera, Camera3D};
use simple_logger::SimpleLogger;
use vent_common::project::VentApplicationProject;

use vent_common::util::crash::init_panic_hook;
use vent_common::window::VentWindow;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent};
use winit::window::WindowBuilder;

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
        let window_builder = WindowBuilder::new().with_title(self.project.name);
        let vent_window = VentWindow::new(window_builder);

        // TODO
        let mut cam = Camera3D::new();

        let mut renderer = RuntimeRenderer::new(
            Dimension::D3,
            &vent_window.window,
            &vent_window.event_loop,
            &mut cam,
        );

        let mut controller = CameraController3D::new(3000.0, 10.0);
        let mut delta_time = 0.0;
        vent_window.event_loop.run(move |event, _, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == vent_window.window.id() => {
                    renderer.progress_event(event);

                    match event {
                        WindowEvent::CloseRequested => control_flow.set_exit(),
                        WindowEvent::MouseInput { button, state, .. } => {
                            controller.process_mouse_input(&vent_window.window, button, state);
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(key),
                                    ..
                                },
                            ..
                        } => {
                            controller.process_keyboard(&mut cam, key, delta_time);
                        }
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(physical_size, &mut cam);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            renderer.resize(new_inner_size, &mut cam);
                        }
                        _ => {}
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => controller.process_mouse_movement(&mut cam, delta.0, delta.1, delta_time),
                Event::RedrawRequested(window_id) if window_id == vent_window.window.id() => {
                    match renderer.render(&vent_window.window, &mut cam) {
                        Ok(d) => delta_time = d,
                        Err(err) => {
                            if err == wgpu::SurfaceError::OutOfMemory {
                                control_flow.set_exit();
                                panic!("{}", format!("{err}"));
                            }
                        }
                    }
                }
                Event::MainEventsCleared => {
                    vent_window.window.request_redraw();
                }
                // ...
                _ => {}
            }
        });
    }
}
