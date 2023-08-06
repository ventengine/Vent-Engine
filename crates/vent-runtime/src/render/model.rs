use glam::{Vec3, Quat};

pub struct Model3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,

    pub rendering_model: vent_assets::Model3D
}

impl  Model3D {
    pub fn new(model: vent_assets::Model3D) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            rendering_model: model
        }
    }
}