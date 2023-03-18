use crate::render::{Dimension, RuntimeRenderer};

use std::time::{Duration, Instant};

use vent_common::components::camera_controller3d::CameraController3D;
use vent_common::entity::camera::{Camera, Camera3D};
use vent_common::project::VentApplicationProject;
use vent_common::render::Renderer;
use vent_common::util::crash::crash;
use vent_common::window::VentWindow;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent};
use winit::window::WindowBuilder;

pub mod render;

pub struct VentApplication {
    project: VentApplicationProject,
}

impl VentApplication {
    pub fn default() {
        env_logger::init();

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

        let mut renderer =
            RuntimeRenderer::new(Dimension::D3, Renderer::new(&vent_window.window), &mut cam);

        let mut controller = CameraController3D::new(50.0, 10.0);

        let mut last = Instant::now();
        let mut delta = Duration::ZERO;
        vent_window.event_loop.run(move |event, _, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == vent_window.window.id() => {
                    match event {
                        WindowEvent::CloseRequested => control_flow.set_exit(),
                        WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(key),
                                ..
                            },
                            ..
                        } => {
                            controller.process_keyboard(&mut cam, key, delta.as_secs_f32());
                        }
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(&vent_window.window, *physical_size, &mut cam);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            renderer.resize(&vent_window.window, **new_inner_size, &mut cam);
                        }
                        _ => {}
                    }
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta, },
                    .. // We're not using device_id currently
                } => {
                   controller.process_mouse(&mut cam, delta.0, delta.1)
                }
                Event::RedrawRequested(window_id) if window_id == vent_window.window.id() => {
                    let now = Instant::now();
                    delta = now - last;
                    last = now;
                    match renderer.render(&vent_window.window, &mut cam) {
                        Ok(_) => {}
                        Err(err) => {
                            if err == wgpu::SurfaceError::OutOfMemory {
                                control_flow.set_exit();
                                crash(format!("{err}"), 101);
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
