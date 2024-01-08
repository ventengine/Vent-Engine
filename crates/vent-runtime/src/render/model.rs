pub struct Entity3D {
    pub model: vent_assets::Model3D,
}

impl Entity3D {
    pub const fn new(model: vent_assets::Model3D) -> Self {
        Self { model }
    }
}
