use std::mem;

use bytemuck::{Pod, Zeroable};

pub mod model;
pub mod pool;
pub mod resource;
pub mod shader;
pub mod texture;

pub trait Asset {}

pub trait Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex for Vertex3D {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

/// A Full Model that will be Loaded from a 3D Model File
/// This is done by Parsing all Essensial Informations like Vertices, Indices, Materials & More
pub struct Model3D {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,

    meshes: Vec<Mesh3D>,
    materials: Vec<wgpu::BindGroup>,
}
/// This is a simple mesh that consists of vertices and indices. It is useful when you need to hard-code 3D data into your application.

/// By using this simple mesh, you can easily define custom shapes or provide default objects for your application. It is particularly handy when you want to avoid loading external model files and instead directly embed the 3D data within your code.

/// Note that this simple mesh implementation does not support advanced features such as normal mapping, skeletal animation, or material properties. It serves as a basic foundation for representing 3D geometry and can be extended or customized according to your specific requirements.

pub struct Mesh3D {
    // Basic
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: u32,

    // Material
    material_id: usize,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
