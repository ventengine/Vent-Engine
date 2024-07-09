use std::time::{Duration, Instant};

use ash::vk::{self};
use serde::{Deserialize, Serialize};
use vent_rendering::instance::VulkanInstance;
use vent_ui::renderer::GuiRenderer;

use crate::project::VentApplicationProject;

use self::camera::{from_dimension, Camera};
use self::d2::Renderer2D;
use self::d3::Renderer3D;
use self::gui::debug_gui::RenderData;

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
        let mut instance = VulkanInstance::new(
            &settings.name,
            settings.version.parse(),
            settings.render_settings.vsync,
            window,
        );
        let dimension = &settings.render_settings.dimension;
        let window_size = window.size();
        let mut camera = from_dimension(window_size.0 as f32 / window_size.1 as f32, dimension);
        let runtime_renderer = RawRuntimeRenderer::new(dimension, &mut instance, camera.as_mut());
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
    fn init(instance: &mut VulkanInstance, camera: &mut dyn Camera) -> Self
    where
        Self: Sized;

    fn resize(
        &mut self,
        instance: &mut VulkanInstance,
        new_size: (u32, u32),
        camera: &mut dyn Camera,
    );

    fn render(
        &mut self,
        instance: &VulkanInstance,
        image_index: u32,
        command_buffer: vk::CommandBuffer,
        camera: &mut dyn Camera,
    );

    fn destroy(&mut self, instance: &VulkanInstance);
}

/// So the idea to split RawRuntimeRenderer and DefaultRuntimeRenderer is, that we can later integrate RawRuntimeRenderer into the Editor view, Just passing VulkanInstance and more
pub struct RawRuntimeRenderer {
    //  gui_renderer: GuiRenderer,
    multi_renderer: Box<dyn Renderer>,
    gui_renderer: GuiRenderer,
    current_data: RenderData,

    current_frames: u32,
    last_fps: Instant,
    delta_time: f32,
}

impl RawRuntimeRenderer {
    pub fn new(
        dimension: &Dimension,
        instance: &mut VulkanInstance,
        camera: &mut dyn Camera,
    ) -> Self {
        let gui_renderer = GuiRenderer::new(instance);
        let multi_renderer: Box<dyn Renderer> = match dimension {
            Dimension::D2 => Box::new(Renderer2D::init(instance, camera)),
            Dimension::D3 => Box::new(Renderer3D::init(instance, camera)),
        };
        //     // TODO
        //     .add_gui(Box::new(DebugGUI::new(unsafe {
        //         instance
        //             .instance
        //             .get_physical_device_properties(instance.physical_device)
        //     })));

        Self {
            multi_renderer,
            gui_renderer,
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
                let command_buffer = instance.command_buffers[image_index as usize];
                unsafe {
                    instance
                        .device
                        .reset_command_buffer(
                            command_buffer,
                            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                        )
                        .unwrap();

                    let info = vk::CommandBufferBeginInfo::default();

                    instance
                        .device
                        .begin_command_buffer(command_buffer, &info)
                        .unwrap();
                }
                self.cmd_renderpass(instance, command_buffer, image_index as usize);

                self.multi_renderer
                    .render(instance, image_index, command_buffer, camera);
                self.gui_renderer.render_text(
                    instance,
                    command_buffer,
                    image_index as usize,
                    "Abc",
                    10.0,
                    10.0,
                    10.0,
                    50,
                );
                let subpass_end_info = vk::SubpassEndInfo::default();
                unsafe {
                    instance
                        .device
                        .cmd_end_render_pass2(command_buffer, &subpass_end_info)
                };
                unsafe { instance.device.end_command_buffer(command_buffer).unwrap() };
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

    fn cmd_renderpass(
        &self,
        instance: &VulkanInstance,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
    ) {
        let render_area = vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(instance.surface_resolution);
        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.2, 0.9, 1.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];

        let info = vk::RenderPassBeginInfo::default()
            .render_pass(instance.render_pass)
            .framebuffer(instance.frame_buffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);
        let subpass_info = vk::SubpassBeginInfo::default().contents(vk::SubpassContents::INLINE);

        unsafe {
            instance
                .device
                .cmd_begin_render_pass2(command_buffer, &info, &subpass_info)
        };
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
        self.gui_renderer.destroy(instance);
        // TODO Egui destroy
    }
}
