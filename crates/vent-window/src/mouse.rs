#[derive(PartialEq, Clone)]
pub enum Key {
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
