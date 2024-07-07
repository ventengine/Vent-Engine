use std::time::{Duration, Instant};

use ash::vk;
use serde::{Deserialize, Serialize};
use vent_rendering::instance::VulkanInstance;

use crate::project::VentApplicationProject;

use self::camera::{from_dimension, Camera};
use self::d2::Renderer2D;
use self::d3::Renderer3D;
use self::gui::debug_gui::{DebugGUI, RenderData};

pub mod camera;
pub mod gui;
pub mod model;

mod model_renderer;

mod d2;
mod d3;

pub(crate) struct DefaultRuntimeRenderer {
    instance: VulkanInstance,
    runtime_renderer: RawRuntimeRenderer,
    pub camera: Box<dyn Camera>,
}

impl DefaultRuntimeRenderer {
    pub(crate) fn new(settings: &VentApplicationProject, window: &vent_window::Window) -> Self {
        let instance = VulkanInstance::new(
            &settings.name,
            settings.version.parse(),
            settings.render_settings.vsync,
            window,
        );
        let dimension = &settings.render_settings.dimension;
        let window_size = window.size();
        let mut camera = from_dimension(window_size.0 as f32 / window_size.1 as f32, dimension);
        let runtime_renderer = RawRuntimeRenderer::new(dimension, &instance, camera.as_mut());
        Self {
            instance,
            runtime_renderer,
            camera,
        }
    }

    pub(crate) fn progress_event(&mut self, event: &vent_window::WindowEvent) {
        self.runtime_renderer.progress_event(event);
    }

    pub(crate) fn render(&mut self) -> f32 {
        self.runtime_renderer
            .render(&mut self.instance, self.camera.as_mut())
    }

    pub(crate) fn resize(&mut self, new_size: (u32, u32)) {
        let old_size = self.instance.surface_resolution;
        if old_size.width == new_size.0 && old_size.height == new_size.1 {
            return;
        }

        log::debug!("Resizing to {:?} ", new_size);
        self.camera
            .recreate_projection(new_size.0 as f32 / new_size.1 as f32);
        self.runtime_renderer
            .resize(&mut self.instance, new_size, self.camera.as_mut());
    }
}

impl Drop for DefaultRuntimeRenderer {
    fn drop(&mut self) {
        self.runtime_renderer.destroy(&self.instance);
    }
}

#[derive(Serialize, Deserialize)]
pub enum Dimension {
    D2,
    D3,
}

pub trait Renderer {
    fn init(instance: &VulkanInstance, camera: &mut dyn Camera) -> Self
    where
        Self: Sized;

    fn resize(
        &mut self,
        instance: &mut VulkanInstance,
        new_size: (u32, u32),
        camera: &mut dyn Camera,
    );

    fn render(&mut self, instance: &VulkanInstance, image_index: u32, camera: &mut dyn Camera);

    fn destroy(&mut self, instance: &VulkanInstance);
}

pub struct RawRuntimeRenderer {
    //  gui_renderer: GuiRenderer,
    multi_renderer: Box<dyn Renderer>,

    current_data: RenderData,

    current_frames: u32,
    last_fps: Instant,
    delta_time: f32,
}

impl RawRuntimeRenderer {
    pub fn new(dimension: &Dimension, instance: &VulkanInstance, camera: &mut dyn Camera) -> Self {
        let multi_renderer: Box<dyn Renderer> = match dimension {
            Dimension::D2 => Box::new(Renderer2D::init(instance, camera)),
            Dimension::D3 => Box::new(Renderer3D::init(instance, camera)),
        };
        // let gui_renderer = GuiRenderer::new()
        //     // TODO
        //     .add_gui(Box::new(DebugGUI::new(unsafe {
        //         instance
        //             .instance
        //             .get_physical_device_properties(instance.physical_device)
        //     })));

        Self {
            multi_renderer,
            //    gui_renderer,
            current_frames: 0,
            current_data: RenderData::default(),
            last_fps: Instant::now(),
            delta_time: 0.0,
        }
    }

    pub fn render(&mut self, instance: &mut VulkanInstance, camera: &mut dyn Camera) -> f32 {
        let frame_start = Instant::now();

        let image = instance.next_image();

        match image {
            Ok((image_index, _)) => {
                self.multi_renderer.render(instance, image_index, camera);
                let result = instance.submit(image_index);
                if let Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR) = result
                {
                    instance.recreate_swap_chain(None);
                }
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                instance.recreate_swap_chain(None);
            }
            Err(_) => {}
        }

        // self.gui_renderer.render(
        //     surface_view,
        //     window,
        //     device,
        //     queue,
        //     &mut encoder2,
        //     &self.current_data,
        // );

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

    pub fn progress_event(&mut self, event: &vent_window::WindowEvent) {
        // self.gui_renderer.progress_event(event);
    }

    pub fn resize(
        &mut self,
        instance: &mut VulkanInstance,
        new_size: (u32, u32),
        camera: &mut dyn Camera,
    ) {
        // Uses the NEW Resized config
        instance.recreate_swap_chain(Some(new_size));
        self.multi_renderer.resize(instance, new_size, camera)
    }

    pub fn destroy(&mut self, instance: &VulkanInstance) {
        self.multi_renderer.destroy(instance);
        // TODO Egui destroy
    }
}
