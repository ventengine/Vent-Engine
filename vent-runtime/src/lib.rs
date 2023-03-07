use crate::render::{Dimension, RuntimeRenderer};

use vent_common::project::VentApplicationProject;
use vent_common::render::Renderer;
use vent_common::window::VentWindow;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

pub mod render;

pub struct VentApplication {
    project: VentApplicationProject,
}

impl VentApplication {
    pub fn new<'a>(project: VentApplicationProject) -> Self {
        Self { project }
    }

    pub fn start(self) {
        let window_builder = WindowBuilder::new().with_title(self.project.name);
        let vent_window = VentWindow::new(window_builder);

        // TODO
        let mut renderer = RuntimeRenderer::new(Dimension::D3, Renderer::new(&vent_window.window));

        vent_window.event_loop.run(move |event, _, control_flow| {
            control_flow.set_wait();

            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == vent_window.window.id() => {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.set_exit(),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(&vent_window.window, *physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            renderer.resize(&vent_window.window, **new_inner_size);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == vent_window.window.id() => {
                    match renderer.render(&vent_window.window) {
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
