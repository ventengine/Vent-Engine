use vent_rendering::instance::VulkanInstance;
use winit::raw_window_handle::DisplayHandle;

use super::{debug_gui::RenderData, GUI};

pub struct EguiRenderer {
    // renderer: egui_winit_ash_integration::Integration,
    context: egui::Context,
    //   state: egui_winit::State,
    guis: Vec<Box<dyn GUI>>,
}

impl EguiRenderer {
    pub fn new() -> Self {
        //let renderer = egui_winit_ash_integration::Integration::new(event_loop, );
        let context = egui::Context::default();
        // let state = egui_winit::State::new(event_loop);
        Self {
            context,
            //   state,
            guis: Vec::new(),
        }
    }

    pub fn render(&mut self, render_data: &RenderData) {
        // let input = self.state.take_egui_input(window);
        // let output = self.context.run(input, |ctx| {
        //     for gui in self.guis.iter_mut() {
        //         gui.update(ctx, render_data);
        //     }
        // });

        // self.state
        //     .handle_platform_output(window, &self.context, output.platform_output);

        // let clipped_meshes = self.context.tessellate(output.shapes);

        // for (texture_id, image_delta) in output.textures_delta.set {
        //     self.renderer
        //         .update_texture(device, queue, texture_id, &image_delta);
        // }

        // let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
        //     size_in_pixels: window.inner_size().into(),
        //     pixels_per_point: self.context.pixels_per_point(),
        // };

        // self.renderer
        //     .update_buffers(device, queue, encoder, &clipped_meshes, &screen_descriptor);

        // let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: Some("GUI Render Pass"),
        //     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //         view,
        //         resolve_target: None,
        //         ops: wgpu::Operations {
        //             load: wgpu::LoadOp::Load,
        //             store: true,
        //         },
        //     })],
        //     depth_stencil_attachment: None,
        // });

        // self.renderer
        //     .render(&mut render_pass, &clipped_meshes, &screen_descriptor);

        // drop(render_pass);

        // for texture_id in output.textures_delta.free {
        //     self.renderer.free_texture(&texture_id);
        // }
    }

    pub fn add_gui(mut self, gui: Box<dyn GUI>) -> Self {
        self.guis.push(gui);
        self
    }

    #[inline]
    #[allow(dead_code)]
    pub fn register_texture(&mut self) {
        // self.renderer.register_native_texture_with_sampler_options(
        //     device,
        //     texture,
        //     sampler_descriptor,
        // )
    }

    pub fn progress_event(&mut self, event: &winit::event::WindowEvent) {
        // let _ = self.state.on_event(&self.context, event);
    }
}
