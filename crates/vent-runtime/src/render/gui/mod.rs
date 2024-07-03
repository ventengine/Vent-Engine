use self::debug_gui::RenderData;

pub mod debug_gui;
pub mod gui_renderer;

pub trait GUI {
    fn update(&mut self, render_data: &RenderData);
}
