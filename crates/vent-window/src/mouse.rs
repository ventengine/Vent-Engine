#[derive(PartialEq, Clone)]
pub enum Button {
    LEFT,
    RIGHT,
    MIDDLE,
    SIDE,
    EXTRA,
    FORWARD,
    BACK,
}

#[derive(PartialEq, Clone)]
pub enum ButtonState {
    Pressed,
    Released,
}
