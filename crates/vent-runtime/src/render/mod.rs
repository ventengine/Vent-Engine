use std::time::{Duration, Instant};

use vent_common::render::WGPURenderer;

use vent_rendering::instance::Instance;
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

pub(crate) struct DefaultRuntimeRenderer {
    instance: Instance,
    runtime_renderer: RawRuntimeRenderer,
}

impl DefaultRuntimeRenderer {
    pub(crate) fn new(
        dimension: Dimension,
        window: &winit::window::Window,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        camera: &mut dyn Camera,
    ) -> Self {
        let instance = Instance::new("TODO", window);
        let runtime_renderer = RawRuntimeRenderer::new(dimension, instance, event_loop, camera);
        Self {
            instance,
            runtime_renderer,
        }
    }

    pub(crate) fn progress_event(&mut self, event: &winit::event::WindowEvent<'_>) {
        self.runtime_renderer.progress_event(event);
    }

    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        camera: &mut dyn Camera,
    ) -> f32 {
        let output = self.wgpu_renderer.surface.get_current_texture().unwrap(); // TODO

        let view: wgpu::TextureView = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Runtime View"),
            ..Default::default()
        });

        let detla = self
            .runtime_renderer
            .render(&view, self.instance, window, camera);

        output.present();

        detla
    }

    pub(crate) fn resize(&mut self, new_size: &PhysicalSize<u32>, camera: &mut dyn Camera) {
        self.runtime_renderer.resize(
            &self.wgpu_renderer.device,
            &self.wgpu_renderer.queue,
            &self.wgpu_renderer.config,
            new_size,
            camera,
        );
    }
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
        camera: &mut dyn Camera,
    ) -> Self
    where
        Self: Sized;

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    );

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        queue: &wgpu::Queue,
        camera: &mut dyn Camera,
    );
}

pub struct RawRuntimeRenderer {
    gui_renderer: EguiRenderer,
    multi_renderer: Box<dyn Renderer>,

    current_data: RenderData,

    current_frames: u32,
    last_fps: Instant,
    delta_time: f32,
}

impl RawRuntimeRenderer {
    pub fn new(
        dimension: Dimension,
        instance: Instance,
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        camera: &mut dyn Camera,
    ) -> Self {
        let multi_renderer: Box<dyn Renderer> = match dimension {
            Dimension::D2 => Box::new(Renderer2D::init(instance, camera)),
            Dimension::D3 => Box::new(Renderer3D::init(instance, camera)),
        };
        let egui = EguiRenderer::new(event_loop, device, surface_format)
            // TODO
            .add_gui(Box::new(DebugGUI::new(adapter.get_info())));

        Self {
            multi_renderer,
            gui_renderer: egui,
            current_frames: 0,
            current_data: RenderData::default(),
            last_fps: Instant::now(),
            delta_time: 0.0,
        }
    }

    pub fn render(&mut self, instance: Instance, window: &Window, camera: &mut dyn Camera) -> f32 {
        let frame_start = Instant::now();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Runtime Render Encoder"),
        });
        // let mut encoder2 = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        //     label: Some("EGUI Render Encoder"),
        // });

        self.multi_renderer
            .render(&mut encoder, surface_view, queue, camera);
        queue.submit(Some(encoder.finish()));

        // self.gui_renderer.render(
        //     surface_view,
        //     window,
        //     device,
        //     queue,
        //     &mut encoder2,
        //     &self.current_data,
        // );

        // queue.submit(Some(encoder2.finish()));

        // TODO
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(offscreen_canvas_setup) = &default_renderer.offscreen_canvas_setup {
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

        self.current_data = self.calc_render_data(frame_start);

        self.delta_time
    }

    fn calc_render_data(&mut self, frame_start: Instant) -> RenderData {
        self.current_frames += 1;

        self.delta_time = frame_start.elapsed().as_secs_f32();

        let now = Instant::now();
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

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        _new_size: &PhysicalSize<u32>,
        camera: &mut dyn Camera,
    ) {
        // Uses the NEW Resized config
        self.multi_renderer.resize(config, device, queue, camera)
    }
}
