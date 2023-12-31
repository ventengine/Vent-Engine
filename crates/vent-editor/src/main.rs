use crate::render::EditorRenderer;

use simple_logger::SimpleLogger;

use vent_common::util::crash::init_panic_hook;
use vent_common::window::VentWindow;
use vent_runtime::render::camera::{Camera, Camera3D};

use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

mod gui;
mod render;

fn main() {
    init_panic_hook();
    #[cfg(not(target_arch = "wasm32"))]
    {
        SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .init()
            .unwrap();
    };

    let window_builder = WindowBuilder::new()
        .with_title(format!("Vent-Editor v{}", env!("CARGO_PKG_VERSION")))
        .with_inner_size(winit::dpi::LogicalSize::new(1400.0, 800.0));
    // TODO
    // .with_window_icon(Some(VentWindow::load_icon(path)));
    let vent_window = VentWindow::new(window_builder);

    let mut camera = Camera3D::new();

    let mut renderer =
        EditorRenderer::new(&vent_window.window, &vent_window.event_loop, &mut camera);
    vent_window
        .event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == vent_window.window.id() => {
                    renderer.egui.progress_event(event);
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            renderer.resize(physical_size, &mut camera);
                        }
                        WindowEvent::RedrawRequested => {
                            renderer.render(&vent_window.window, &mut camera);
                        }
                        // WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        //     // new_inner_size is &mut so w have to dereference it twice
                        //     renderer.resize(new_inner_size, &mut camera);
                        // }
                        _ => {}
                    }
                }
                // ...
                _ => {}
            }
        })
        .expect("Window Event Loop Error");
}
