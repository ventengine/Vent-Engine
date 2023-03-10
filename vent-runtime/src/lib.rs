use crate::render::{Dimension, RuntimeRenderer};
use std::time::Instant;

use vent_common::component::components::camera_controller3d::CameraController3D;
use vent_common::entities::camera::{Camera, Camera3D};
use vent_common::project::VentApplicationProject;
use vent_common::render::Renderer;
use vent_common::window::VentWindow;
use winit::event::{Event, KeyboardInput, WindowEvent};
use winit::window::WindowBuilder;

pub mod render;

pub struct VentApplication {
    project: VentApplicationProject,
}

impl VentApplication {
    pub fn new(project: VentApplicationProject) -> Self {
        Self { project }
    }

    pub fn start(self) {
        let window_builder = WindowBuilder::new().with_title(self.project.name);
        let vent_window = VentWindow::new(window_builder);

        // TODO
        let mut cam = Camera3D::new();

        let mut renderer = RuntimeRenderer::new(Dimension::D3, Renderer::new(&vent_window.window), &mut cam);

        let mut controller = CameraController3D::new(1.0, 10.0);

        let mut last = Instant::now();
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
                                state,
                                virtual_keycode: Some(key),
                                ..
                            },
                            ..
                        } => {
                            controller.process_keyboard(key, *state);
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
                Event::RedrawRequested(window_id) if window_id == vent_window.window.id() => {
                    let now = Instant::now();
                    let delta = now - last;
                    controller.update_camera(&mut cam, delta);
                    last = now;
                    match renderer.render(&vent_window.window, &mut cam) {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        // Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => render.resize(render.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => control_flow.set_exit(),
                        _ => {}
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
