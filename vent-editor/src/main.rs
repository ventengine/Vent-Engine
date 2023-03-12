use crate::render::EditorRenderer;
use std::path::Path;
use vent_common::entities::camera::{Camera, Camera3D};
use vent_common::window::VentWindow;
use wgpu::SurfaceError;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

mod render;

fn main() {
    env_logger::init();

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/textures/icon/icon64.png"
    );

    let window_builder = WindowBuilder::new()
        .with_title(format!("Vent-Editor v{}", env!("CARGO_PKG_VERSION")))
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
                let _ = renderer.egui.state.on_event(&renderer.egui.context, event);
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
                        renderer.resize(&vent_window.window, *physical_size, &mut camera);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        renderer.resize(&vent_window.window, **new_inner_size, &mut camera);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == vent_window.window.id() => {
                match renderer.render(&vent_window.window, &mut camera) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    // Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => render.resize(render.size),
                    // The system is out of memory, we should probably quit
                    Err(SurfaceError::OutOfMemory) => control_flow.set_exit(),
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
