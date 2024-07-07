pub mod font;
pub mod renderer;
pub mod widgets;

pub trait GUI {
    fn update(&mut self);
}
