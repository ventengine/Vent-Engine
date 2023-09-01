use std::any::Any;
use std::rc::Rc;
use std::time::{Duration, Instant};

use vent_common::render::DefaultRenderer;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use self::camera::Camera;
use self::d2::Renderer2D;
use self::d3::Renderer3D;
use self::gui::debug_gui::{DebugGUI, RenderData};
use self::gui::gui_renderer::EguiRenderer;

pub mod camera;
pub mod gui;
pub mod model;

mod model_renderer;

mod d2;
mod d3;

pub struct RuntimeRenderer {
    default_renderer: DefaultRenderer,
    gui_renderer: EguiRenderer,
    multi_renderer: Box<dyn Renderer>,

    depth_view: wgpu::TextureView,

    current_data: RenderData,

    current_frames: u32,
    last_delta: Instant,
    last_fps: Instant,
    delta_time: f32,
}
pub enum Dimension {
    D2,
    D3,
}

pub trait Renderer {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Any,
    ) -> Self
    where
        Self: Sized;

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        queue: &wgpu::Queue,
    );
}

impl RuntimeRenderer {
    pub fn new(
        dimension: Dimension,
        window: &winit::window::Window,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        mut camera: &mut dyn Camera,
    ) -> Self {
        let default_renderer = DefaultRenderer::new(window);
        let egui = EguiRenderer::new(
            event_loop,
            &default_renderer.device,
            default_renderer.caps.formats[0],
        )
        // TODO
        .add_gui(Box::new(DebugGUI::new(default_renderer.adapter.get_info())));
        let depth_view = vent_assets::Texture::create_depth_view(
            &default_renderer.device,
            default_renderer.config.width,
            default_renderer.config.height,
            Some("Depth Buffer"),
        );

        Self {
            multi_renderer: match dimension {
                Dimension::D2 => Box::new(Renderer2D::init(
                    &default_renderer.config,
                    &default_renderer.device,
                    &default_renderer.queue,
                    &mut camera,
                )),
                Dimension::D3 => Box::new(Renderer3D::init(
                    &default_renderer.config,
                    &default_renderer.device,
                    &default_renderer.queue,
                    &mut camera,
                )),
            },
            gui_renderer: egui,
            default_renderer,
            depth_view,

            current_frames: 0,
            current_data: RenderData::default(),
            last_delta: Instant::now(),
            last_fps: Instant::now(),
            delta_time: 0.0,
        }
    }

    pub fn render(&mut self, window: &Window) -> Result<f32, SurfaceError> {
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
        let mut encoder2 =
            self.default_renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Runtime Render Encoder"),
                });

        self.multi_renderer.render(
            &mut encoder,
            &view,
            &self.depth_view,
            &self.default_renderer.queue,
        );
        self.default_renderer.queue.submit(Some(encoder.finish()));

        self.gui_renderer.render(
            &view,
            window,
            &self.default_renderer.device,
            &self.default_renderer.queue,
            &mut encoder2,
            &self.current_data,
        );

        self.default_renderer.queue.submit(Some(encoder2.finish()));

        output.present();

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(offscreen_canvas_setup) = &self.default_renderer.offscreen_canvas_setup {
                let image_bitmap = offscreen_canvas_setup
                    .offscreen_canvas
                    .transfer_to_image_bitmap()
                    .expect("couldn't transfer offscreen canvas to image bitmap.");
                offscreen_canvas_setup
                    .bitmap_renderer
                    .transfer_from_image_bitmap(&image_bitmap);

                log::info!("Transferring OffscreenCanvas to ImageBitmapRenderer");
            }
        }

        self.current_data = self.calc_render_data();

        Ok(self.delta_time)
    }

    fn calc_render_data(&mut self) -> RenderData {
        self.current_frames += 1;

        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_delta).as_secs_f32();
        self.last_delta = now;

        if now - self.last_fps >= Duration::from_secs(1) {
            self.current_data.fps = self.current_frames;
            self.current_frames = 0;
            self.last_fps = now;
        }

        RenderData {
            fps: self.current_data.fps,
            frame_time: self.delta_time,
        }
    }

    pub fn progress_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        self.gui_renderer.progress_event(event);
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        // Its Important to resize Default Renderer first
        self.default_renderer.resize(new_size);
        self.depth_view = vent_assets::Texture::create_depth_view(
            &self.default_renderer.device,
            self.default_renderer.config.width,
            self.default_renderer.config.height,
            Some("Depth Buffer"),
        );

        self.multi_renderer.resize(
            &self.default_renderer.config,
            &self.default_renderer.device,
            &self.default_renderer.queue,
        )
    }
}
