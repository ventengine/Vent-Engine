use ash::vk;
use vent_rendering::buffer::VulkanBuffer;

pub mod model;
pub mod pool;
pub mod shader;

pub trait Asset {}

/// A Full Model that can be Loaded from a 3D Model File
/// This is done by Parsing all Essensial Informations like Vertices, Indices, Materials & More
pub struct Model3D {
    meshes: Vec<Mesh3D>,
}
/// This is a simple mesh that consists of vertices and indices. It is useful when you need to hard-code 3D data into your application.

/// By using this simple mesh, you can easily define custom shapes or provide default objects for your application. It is particularly handy when you want to avoid loading external model files and instead directly embed the 3D data within your code.

/// Note that this simple mesh implementation does not support advanced features such as normal mapping, skeletal animation, or material properties. It serves as a basic foundation for representing 3D geometry and can be extended or customized according to your specific requirements.

pub struct Mesh3D {
    // Basic
    vertex_buf: VulkanBuffer,
    index_buf: VulkanBuffer,
    index_count: u32,
    descriptor_sets: Option<Vec<vk::DescriptorSet>>,
    buffers: Option<Vec<VulkanBuffer>>,
}
