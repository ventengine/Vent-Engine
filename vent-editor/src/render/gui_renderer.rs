use imgui_wgpu::RendererConfig;
use imgui_winit_support::WinitPlatform;
use vent_common::render::{DefaultRenderer, Renderer};
use wgpu::{Device, Queue, RenderPass, SurfaceConfiguration, SurfaceError};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct ImGUIRenderer {
    pub context: imgui::Context,
    pub winit_platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

impl ImGUIRenderer {
    pub(crate) fn new(
        window: &Window,
        queue: &Queue,
        device: &Device,
        config: &SurfaceConfiguration,
    ) -> Self {
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);

        let mut winit_platform = WinitPlatform::init(&mut imgui_context);
        winit_platform.attach_window(
            imgui_context.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Rounded,
        );

        imgui_context
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;

        let renderer_config = RendererConfig {
            texture_format: config.format,
            ..Default::default()
        };

        let renderer =
            imgui_wgpu::Renderer::new(&mut imgui_context, device, queue, renderer_config);

        ImGUIRenderer {
            context: imgui_context,
            winit_platform,
            renderer,
        }
    }

    pub fn pre_render(&mut self, window: &Window) {
        self.winit_platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame");
        let ui = self.context.frame();
        ui.show_demo_window(&mut true);
        self.winit_platform.prepare_render(ui, window);
    }

    pub fn post_render<'r>(
        &'r mut self,
        _window: &Window,
        queue: &Queue,
        device: &Device,
        render_pass: &mut RenderPass<'r>,
    ) -> Result<(), SurfaceError> {
        self.renderer
            .render(self.context.render(), queue, device, render_pass)
            .expect("Rendering failed");
        Ok(())
    }

    pub fn resize(renderer: &mut DefaultRenderer, window: &Window, new_size: PhysicalSize<u32>) {
        Renderer::resize(renderer, window, new_size);
    }
}
