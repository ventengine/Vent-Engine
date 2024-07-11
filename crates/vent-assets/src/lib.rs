use ash::vk::{self};
use gltf::material::AlphaMode;
use vent_rendering::{image::VulkanImage, mesh::Mesh3D};

mod image;
pub mod io;
pub mod model;

pub trait Asset: Send + Sync + 'static {}

/// A Full Model/Scene that can be Loaded from a 3D Model File
/// This is done by Parsing all Essensial Informations like Vertices, Indices, Materials & More
pub struct Model3D {
    pub pipelines: Vec<ModelPipeline>,
    pub materials: Vec<Material>,

    pub descriptor_pool: vk::DescriptorPool,

    pub position: [f32; 3], // Default: 0.0, 0.0, 0.0
    pub rotation: [f32; 4], // Default: 0.0, 0.0, 0.0, 1.0
    pub scale: [f32; 3],    // Default: 1.0, 1.0, 1.0
}

/// Often we must create new Pipelines for Materials/Meshes
pub struct ModelPipeline {
    pub pipeline: vk::Pipeline,
    pub materials: Vec<ModelMaterial>,
}

pub struct ModelMaterial {
    pub material_index: usize,
    // So every App is Specfic and you will need to create your own DescriptorSet's out of this
    // We only binding them
    pub meshes: Vec<Mesh3D>,
}

pub struct Material {
    pub diffuse_texture: VulkanImage,
    pub descriptor_set: Option<Vec<vk::DescriptorSet>>,
    pub base_color: [f32; 4],
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
    pub alpha_cut: f32,
}
