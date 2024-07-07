use ash::vk;

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
    properties: vk::PhysicalDeviceProperties,
}

impl DebugGUI {
    pub const fn new(properties: vk::PhysicalDeviceProperties) -> Self {
        Self { properties }
    }
}

// impl GUI for DebugGUI {
//     fn update(&mut self, render_data: &RenderData) {}
//     // fn update(&mut self, ctx: &egui::Context, render_data: &RenderData) {
//     //     egui::Window::new("Debug").show(ctx, |ui| {
//     //         ui.label(format!("FPS: {}", render_data.fps));
//     //         ui.label(format!("Frame Time: {}ms", render_data.frame_time));

//     //         ui.label(format!("Device: {:?}", self.properties.device_name));
//     //         ui.label(format!("Device Type: {:?}", self.properties.device_type));
//     //         ui.label(format!(
//     //             "Driver: {}, API Version: {}",
//     //             self.properties.driver_version, self.properties.api_version
//     //         ));
//     //     });
//     // }
// }
