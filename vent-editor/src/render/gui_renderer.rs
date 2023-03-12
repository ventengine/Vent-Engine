pub struct EguiRenderer {
    renderer: egui_wgpu::Renderer,
    pub context: egui::Context,
    pub state: egui_winit::State,
}

impl EguiRenderer {
    pub fn new<T>(
        event_loop: &winit::event_loop::EventLoopWindowTarget<T>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1);
        let context = egui::Context::default();
        let state = egui_winit::State::new(event_loop);
        Self {
            renderer,
            context,
            state,
        }
    }

    pub fn render<'r>(
        &'r mut self,
        renderpass: &mut wgpu::RenderPass<'r>,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.context.begin_frame(self.state.take_egui_input(window));

        egui::Window::new("UwU").show(&self.context, |ui| {
            ui.heading("My egui Application");
        });

        let output = self.context.end_frame();

        self.state
            .handle_platform_output(window, &self.context, output.platform_output);

        let clipped_meshes = self.context.tessellate(output.shapes);

        for (texture_id, image_delta) in output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, texture_id, &image_delta);
        }

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [
                window.inner_size().width as _,
                window.inner_size().height as _,
            ],
            pixels_per_point: window.scale_factor() as _,
        };

        self.renderer
            .update_buffers(device, queue, encoder, &clipped_meshes, &screen_descriptor);

        self.renderer
            .render(renderpass, &clipped_meshes, &screen_descriptor);
    }

    #[inline]
    pub fn register_texture(
        &mut self,
        _device: &wgpu::Device,
        _texture: &wgpu::TextureView,
        _filter: wgpu::FilterMode,
    ) -> egui::TextureId {
        //   self.renderer.update_egui_texture_from_wgpu_texture(device, texture, filter)
        todo!()
    }

    #[inline]
    pub fn atlas_id(&self) -> egui::TextureId {
        self.atlas_id()
    }
}
