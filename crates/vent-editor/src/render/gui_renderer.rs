use crate::gui::EditorGUI;

pub struct EguiRenderer {
    renderer: egui_wgpu::Renderer,
    gui: EditorGUI,
    context: egui::Context,
    state: egui_winit::State,
}

impl EguiRenderer {
    pub fn new(
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1);
        let context = egui::Context::default();
        let state = egui_winit::State::new(event_loop);
        Self {
            renderer,
            gui: EditorGUI::new(),
            context,
            state,
        }
    }

    pub fn render<'rp>(
        &'rp mut self,
        rpass: &mut wgpu::RenderPass<'rp>,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        //  self.context.begin_frame(self.state.take_egui_input(window));
        let input = self.state.take_egui_input(window);
        let output = self.context.run(input, |ctx| {
            self.gui.update(ctx);
        });

        self.state
            .handle_platform_output(window, &self.context, output.platform_output);

        let clipped_meshes = self.context.tessellate(output.shapes);

        for (texture_id, image_delta) in output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, texture_id, &image_delta);
        }

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: window.inner_size().into(),
            pixels_per_point: window.scale_factor() as _,
        };

        self.renderer
            .update_buffers(device, queue, encoder, &clipped_meshes, &screen_descriptor);

        self.renderer
            .render(rpass, &clipped_meshes, &screen_descriptor);
    }

    #[inline]
    #[allow(dead_code)]
    pub fn register_texture(
        &mut self,
        _device: &wgpu::Device,
        _texture: &wgpu::TextureView,
        _filter: wgpu::FilterMode,
    ) -> egui::TextureId {
        //   self.renderer.update_egui_texture_from_wgpu_texture(device, texture, filter)
        todo!()
    }

    pub fn progress_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        let _ = self.state.on_event(&self.context, event);
    }
}
