use super::GUI;

pub struct RenderData {
    pub fps: u32,
    pub frame_time: f32,
}

impl Default for RenderData {
    fn default() -> Self {
        Self {
            fps: 1,
            frame_time: 1.0,
        }
    }
}

pub struct DebugGUI {
    adapter: wgpu::AdapterInfo,
}

impl DebugGUI {
    pub fn new(adapter: wgpu::AdapterInfo) -> Self {
        Self { adapter }
    }
}

impl GUI for DebugGUI {
    fn update(&mut self, ctx: &egui::Context, render_data: &RenderData) {
        egui::Window::new("Debug").show(ctx, |ui| {
            ui.label(format!("FPS: {}", render_data.fps));
            ui.label(format!("Frame Time: {}", render_data.frame_time));

            // WGPU 0.17  ui.label(format!("API: {}", self.adapter.backend));
            ui.label(format!("Device: {}", self.adapter.name));
            // WGPU 0.17  ui.label(format!("Device Type: {}", self.adapter.device_type));
            ui.label(format!("Device Driver: {}, Info {}", self.adapter.driver, self.adapter.driver_info));
        });
    }
}
