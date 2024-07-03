use vent_math::scalar::{mat4::Mat4, quat::Quat};

pub struct Entity3D {
    pub model: vent_assets::Model3D,
    pub transformation_matrix: Mat4,
}

impl Entity3D {
    pub fn new(model: vent_assets::Model3D) -> Self {
        Self {
            transformation_matrix: Self::calc_trans_matrix(&model),
            model,
        }
    }

    pub fn calc_trans_matrix(model: &vent_assets::Model3D) -> Mat4 {
        let rotation_quat = Quat::from_scaled_axis(Quat::from_array(model.rotation).xyz());
        Mat4::from_scale_rotation_translation(
            model.scale.into(),
            rotation_quat,
            model.position.into(),
        )
    }
}
