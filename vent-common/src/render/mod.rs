pub mod model;

use pollster::block_on;
use wgpu::{
    Adapter, Device, Queue, Surface, SurfaceCapabilities,
    SurfaceConfiguration, SurfaceError,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use bytemuck::{Pod, Zeroable};
use log::debug;
#[cfg(target_arch = "wasm32")]
use std::str::FromStr;

use crate::util::crash::crash;
#[cfg(target_arch = "wasm32")]
use web_sys::{ImageBitmapRenderingContext, OffscreenCanvas};

#[cfg(target_arch = "wasm32")]
/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let mut pair = pair.split('=');
        let key = pair.next()?;
        let value = pair.next()?;

        if key == search_key {
            return Some(value);
        }
    }

    None
}

#[cfg(target_arch = "wasm32")]
pub struct OffscreenCanvasSetup {
    pub offscreen_canvas: OffscreenCanvas,
    pub bitmap_renderer: ImageBitmapRenderingContext,
}

pub struct DefaultRenderer {
    pub surface: Surface,
    pub device: Device,
    pub adapter: Adapter,
    pub queue: Queue,

    pub config: SurfaceConfiguration,
    pub caps: SurfaceCapabilities,

    #[cfg(target_arch = "wasm32")]
    pub offscreen_canvas_setup: Option<OffscreenCanvasSetup>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex3D {
    pub _pos: [f32; 3],
    pub _tex_coord: [f32; 2],
}

pub trait Renderer {
    fn new(window: &Window) -> Self;
    fn render(&mut self, window: &Window, renderer: &DefaultRenderer) -> Result<(), SurfaceError>;
    fn resize(&mut self, window: &Window, new_size: PhysicalSize<u32>);
}

impl Renderer for DefaultRenderer {
    fn new(window: &Window) -> Self {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler,
        });

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let query_string = web_sys::window().unwrap().location().search().unwrap();
            let level: log::Level = parse_url_query_string(&query_string, "RUST_LOG")
                .and_then(|x| x.parse().ok())
                .unwrap_or(log::Level::Error);
            console_log::init_with_level(level).expect("could not initialize logger");
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            // On wasm, append the canvas to the document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
        }

        #[cfg(target_arch = "wasm32")]
        let mut offscreen_canvas_setup: Option<OffscreenCanvasSetup> = None;
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;

            let query_string = web_sys::window().unwrap().location().search().unwrap();
            if let Some(offscreen_canvas_param) =
                parse_url_query_string(&query_string, "offscreen_canvas")
            {
                if FromStr::from_str(offscreen_canvas_param) == Ok(true) {
                    log::info!("Creating OffscreenCanvasSetup");

                    let offscreen_canvas =
                        OffscreenCanvas::new(1024, 768).expect("couldn't create OffscreenCanvas");

                    let bitmap_renderer = window
                        .canvas()
                        .get_context("bitmaprenderer")
                        .expect("couldn't create ImageBitmapRenderingContext (Result)")
                        .expect("couldn't create ImageBitmapRenderingContext (Option)")
                        .dyn_into::<ImageBitmapRenderingContext>()
                        .expect("couldn't convert into ImageBitmapRenderingContext");

                    offscreen_canvas_setup = Some(OffscreenCanvasSetup {
                        offscreen_canvas,
                        bitmap_renderer,
                    })
                }
            }
        };

        let surface = unsafe {
            #[cfg(any(not(target_arch = "wasm32"), target_os = "emscripten"))]
            let surface = match instance.create_surface(&window) {
                Ok(t) => t,
                Err(e) => {
                    crash(format!("{e}"), 102);
                    panic!()
                }
            };
            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            let surface = {
                if let Some(offscreen_canvas_setup) = &offscreen_canvas_setup {
                    log::info!("Creating surface from OffscreenCanvas");
                    instance.create_surface_from_offscreen_canvas(
                        &offscreen_canvas_setup.offscreen_canvas,
                    )
                } else {
                    instance.create_surface(&window)
                }
            }
            .unwrap();

            surface
        };
        let adapter = block_on(wgpu::util::initialize_adapter_from_env_or_default(
            &instance,
            backends,
            Some(&surface),
        ))
        .expect("No suitable GPU adapters found on the system!");

        #[cfg(not(target_arch = "wasm32"))]
        {
            let adapter_info = adapter.get_info();
            debug!("GPU {} {:?}", adapter_info.name, adapter_info.device_type);
            debug!(
                "Software {:?} {}",
                adapter_info.backend, adapter_info.driver_info
            );
        }

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = match block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )) {
            Ok(t) => t,
            Err(e) => {
                crash(format!("{e}"), 102);
                panic!()
            }
        };

        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Surface isn't supported by the adapter.");
        surface.configure(&device, &config);
        let caps = surface.get_capabilities(&adapter);
        Self {
            surface,
            device,
            adapter,
            queue,
            config,
            caps,

            #[cfg(target_arch = "wasm32")]
            offscreen_canvas_setup,
        }
    }

    fn render(
        &mut self,
        _window: &Window,
        _renderer: &DefaultRenderer,
    ) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn resize(&mut self, _window: &Window, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }
}
