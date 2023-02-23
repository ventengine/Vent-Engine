use pollster::block_on;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, SurfaceError};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct DefaultRenderer {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,

    pub config: SurfaceConfiguration,
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

        let surface = unsafe {
            #[cfg(any(not(target_arch = "wasm32"), target_os = "emscripten"))]
            let surface = instance.create_surface(&window).unwrap();
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
            println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
        }

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = block_on(adapter.request_device(
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
        ))
        .expect("Unable to find a suitable GPU adapter!");

        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Surface isn't supported by the adapter.");
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
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
