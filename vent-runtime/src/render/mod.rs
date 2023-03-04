use std::f32::consts;
use vent_common::render::{DefaultRenderer, Renderer};
use wgpu::{CommandEncoder, SurfaceError, SurfaceTexture, TextureView};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use bytemuck::{Pod, Zeroable};

pub struct RuntimeRenderer {
    default_renderer: DefaultRenderer,
}

pub enum Dimension {
    D2,
    D3
}

impl RuntimeRenderer {
    pub(crate) fn new(dimension: Dimension, window: &Window) -> Self {
        let renderer = Renderer::new(window);
        Self {
            default_renderer: renderer,
        }
    }

    pub fn render(&self, _window: &Window) -> Result<(), SurfaceError> {
        let output = self.default_renderer.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Runtime View"),
            ..Default::default()
        });

        let mut encoder =
            self.default_renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Runtime Render Encoder"),
                });

        Self::render_from(&_window, &mut encoder, &view).expect("Failed to Render Runtime");

        self.default_renderer
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn render_from(
        _window: &winit::window::Window,
        encoder: &mut CommandEncoder,
        view: &TextureView,
    ) -> Result<(), SurfaceError> {
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Runtime Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.4,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }
        Ok(())
    }

    pub fn resize(&mut self, window: &Window, new_size: PhysicalSize<u32>) {
        Self::resize_from(&mut self.default_renderer, window, new_size)
    }

    pub fn resize_from(
        renderer: &mut DefaultRenderer,
        window: &Window,
        new_size: PhysicalSize<u32>,
    ) {
        Renderer::resize(renderer, window, new_size);
    }
}

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex3D {
    Vertex3D {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices() -> (Vec<Vertex3D>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex3D {
    _pos: [f32; 3],
    _tex_coord: [f32; 2],
}

pub trait MultiDimensionRenderer {
    // TODO
    fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(1.5f32, -5.0, 3.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );
        projection * view
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
         uniform_buf: wgpu::Buffer,
    ) {
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        queue.write_buffer(&uniform_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    fn render();


}

pub struct Renderer3D {}

impl MultiDimensionRenderer for Renderer3D {
    fn render() {
        todo!()
    }
}
