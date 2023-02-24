use egui::{PlatformOutput, TextureId};
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform::{Platform, PlatformDescriptor};
use vent_common::render::{DefaultRenderer, Renderer};
use wgpu::{CommandEncoder, Device, FilterMode, Queue, RenderPass, SurfaceConfiguration, SurfaceError, TextureView};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct EguiRenderer {
    pub platform: Platform,
    renderer: egui_wgpu_backend::RenderPass,
}

impl EguiRenderer {
    pub fn new(window: &Window, device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: window.inner_size().width as _,
            physical_height: window.inner_size().height as _,
            scale_factor: window.scale_factor(),
            font_definitions: Default::default(),
            style: Default::default(),
        });

        let mut renderer = egui_wgpu_backend::RenderPass::new(device, surface_format, 1);
        Self { platform, renderer }
    }

    pub fn render<'a>(
        &'a mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.platform.begin_frame();

        egui::Window::new("UwU").show(&self.platform.context(), |ui| {
            ui.heading("My egui Application");
        });

        let full_output = self.platform.end_frame(Some(window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);

        let screen_descriptor = ScreenDescriptor {
            physical_width: window.inner_size().width as _,
            physical_height: window.inner_size().height as _,
            scale_factor: window.scale_factor() as _
        };

        let texture_delta = full_output.textures_delta;
        self.renderer.add_textures(device, queue, &texture_delta).expect("Failed to add textures");

        self.renderer.update_buffers(device, queue, &paint_jobs, &screen_descriptor);
        self.renderer
            .execute(encoder, texture_view, &paint_jobs, &screen_descriptor, None)
            .expect("Failed to execute render pass");
    }

    #[inline]
    pub fn register_texture(&mut self, device: &Device, texture: &TextureView, filter: FilterMode) -> TextureId {
        self.renderer.egui_texture_from_wgpu_texture(device, texture, filter)
    }

    #[inline]
    pub fn atlas_id(&self) -> TextureId {
        self.atlas_id()
    }
}