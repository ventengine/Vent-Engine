use ash::vk;
use gltf::material::AlphaMode;
use vent_rendering::{buffer::VulkanBuffer, image::VulkanImage};

pub mod model;
pub mod pool;
pub mod shader;

pub trait Asset {}

/// A Full Model/Scene that can be Loaded from a 3D Model File
/// This is done by Parsing all Essensial Informations like Vertices, Indices, Materials & More
pub struct Model3D {
    pub pipelines: Vec<ModelPipeline>,

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
    pub material: Material,
    // So every App is Specfic and you will need to create your own DescriptorSet's out of this
    // We only binding them
    pub descriptor_set: Option<Vec<vk::DescriptorSet>>,
    pub meshes: Vec<Mesh3D>,
}

/// This is a simple mesh that consists of vertices and indices. It is useful when you need to hard-code 3D data into your application.

/// By using this simple mesh, you can easily define custom shapes or provide default objects for your application. It is particularly handy when you want to avoid loading external model files and instead directly embed the 3D data within your code.

/// Note that this simple mesh implementation does not support advanced features such as normal mapping, skeletal animation, or material properties. It serves as a basic foundation for representing 3D geometry and can be extended or customized according to your specific requirements.

pub struct Mesh3D {
    // Basic
    vertex_buf: VulkanBuffer,
    index_buf: VulkanBuffer,
    index_count: u32,
}

pub struct Material {
    pub diffuse_texture: VulkanImage,
    pub base_color: [f32; 4],
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
    pub alpha_cut: f32,
}
