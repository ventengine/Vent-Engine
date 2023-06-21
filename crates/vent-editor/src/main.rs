use crate::render::EditorRenderer;

use std::path::Path;
use vent_common::entity::camera::{Camera, Camera3D};
use vent_common::util::crash::init_panic_hook;
use vent_common::window::VentWindow;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

mod gui;
mod render;

fn main() {
    init_panic_hook();
    env_logger::init();

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/textures/icon/icon64.png"
    );

    let window_builder = WindowBuilder::new()
        .with_title(format!("Vent-Editor v{}", env!("CARGO_PKG_VERSION")))
        .with_inner_size(winit::dpi::LogicalSize::new(1400.0, 800.0))
        // TODO
        .with_window_icon(Some(VentWindow::load_icon(Path::new(path))));
    let vent_window = VentWindow::new(window_builder);

    let mut camera = Camera3D::new();

    let mut renderer =
        EditorRenderer::new(&vent_window.window, &vent_window.event_loop, &mut camera);
    vent_window.event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == vent_window.window.id() => {
                renderer.egui.progress_event(event);
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
                        renderer.resize(*physical_size, &mut camera);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        renderer.resize(**new_inner_size, &mut camera);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == vent_window.window.id() => {
                match renderer.render(&vent_window.window, &mut camera) {
                    Ok(_) => {}
                    Err(err) => match err {
                        wgpu::SurfaceError::OutOfMemory => {
                            control_flow.set_exit();
                            panic!("{}", format!("{err}"));
                        }
                        wgpu::SurfaceError::Lost => renderer.resize_current(&mut camera),
                        _ => {}
                    },
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
