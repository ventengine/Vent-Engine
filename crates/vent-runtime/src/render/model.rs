use glam::{Quat, Vec3};

pub struct Entity3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,

    pub rendering_model: vent_assets::Model3D,
}

impl Entity3D {
    pub fn new(model: vent_assets::Model3D) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            rendering_model: model,
        }
    }
}
