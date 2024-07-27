use std::mem::offset_of;

use ash::vk;

#[derive(Clone, Copy, PartialEq)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

#[derive(Clone, Copy, PartialEq)]
pub struct VertexPos3D {
    pub position: [f32; 3],
}

#[derive(Clone, Copy, PartialEq)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: u32,
}

impl Vertex3D {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
    pub fn input_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            // offset_of macro got stabilized in rust 1.77
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, tex_coord) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(2)
                .binding(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, normal) as u32),
        ]
    }
}

impl VertexPos3D {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
    pub fn input_descriptions() -> [vk::VertexInputAttributeDescription; 1] {
        [
            // offset_of macro got stabilized in rust 1.77
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, position) as u32),
        ]
    }
}

impl Vertex2D {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
    pub fn input_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            // offset_of macro got stabilized in rust 1.77
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, tex_coord) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(2)
                .binding(0)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(offset_of!(Self, color) as u32),
        ]
    }
}
