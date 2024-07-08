use std::{
    mem::{self, offset_of},
    os::raw::c_void,
};

use ash::vk;
use buffer::VulkanBuffer;
use bytemuck::cast_slice;
use ordered_float::OrderedFloat;

pub mod allocator;
pub mod buffer;
mod cache;
mod debug;
pub mod image;
pub mod instance;
pub mod mesh;
pub mod pipeline;
mod surface;

pub const DEFAULT_TEXTURE_FILTER: vk::Filter = vk::Filter::LINEAR;
pub const DEFAULT_MIPMAP_MODE: vk::SamplerMipmapMode = vk::SamplerMipmapMode::LINEAR;

const DEFAULT_FENCE_TIMEOUT: u64 = 100000000000;

// Used for caching
#[derive(PartialEq, Eq, Hash)]
pub struct MaterialPipelineInfo {
    pub mode: vk::PrimitiveTopology,
    pub alpha_cut: Option<OrderedFloat<f32>>, // Default 0.5
    pub double_sided: bool,
}

// Used for caching
#[derive(PartialEq, Eq, Hash)]
pub struct SamplerInfo {
    pub mag_filter: vk::Filter,
    pub min_filter: vk::Filter,
    pub mipmap_mode: vk::SamplerMipmapMode,
    pub address_mode_u: vk::SamplerAddressMode,
    pub address_mode_v: vk::SamplerAddressMode,
    pub address_mode_w: vk::SamplerAddressMode,
    pub mip_lod_bias: OrderedFloat<f32>,
    pub anisotropy_enable: bool,
    pub max_anisotropy: OrderedFloat<f32>,
    pub compare_enable: bool,
    pub compare_op: vk::CompareOp,
    pub min_lod: OrderedFloat<f32>,
    pub max_lod: OrderedFloat<f32>,
    pub border_color: vk::BorderColor,
    pub unnormalized_coordinates: bool,
}

impl SamplerInfo {
    pub fn to_vk(&self) -> vk::SamplerCreateInfo<'static> {
        vk::SamplerCreateInfo::default()
            .mag_filter(self.mag_filter)
            .min_filter(self.min_filter)
            .mipmap_mode(self.mipmap_mode)
            .address_mode_u(self.address_mode_u)
            .address_mode_v(self.address_mode_v)
            .address_mode_w(self.address_mode_w)
            .mip_lod_bias(*self.mip_lod_bias)
            .anisotropy_enable(self.anisotropy_enable)
            .max_anisotropy(*self.max_anisotropy)
            .compare_enable(self.compare_enable)
            .compare_op(self.compare_op)
            .min_lod(*self.min_lod)
            .max_lod(*self.max_lod)
            .border_color(self.border_color)
            .unnormalized_coordinates(self.unnormalized_coordinates)
    }
}

impl Default for SamplerInfo {
    fn default() -> Self {
        Self {
            mag_filter: DEFAULT_TEXTURE_FILTER,
            min_filter: DEFAULT_TEXTURE_FILTER,
            mipmap_mode: DEFAULT_MIPMAP_MODE,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            mip_lod_bias: Default::default(),
            anisotropy_enable: Default::default(),
            max_anisotropy: Default::default(),
            compare_enable: false,
            compare_op: Default::default(),
            min_lod: Default::default(),
            max_lod: Default::default(),
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

#[derive(Clone, Copy, PartialEq)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: u32,
}

pub enum Indices {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Indices {
    /// Returns the number of indices.
    pub fn len(&self) -> usize {
        match self {
            Indices::U8(vec) => vec.len(),
            Indices::U16(vec) => vec.len(),
            Indices::U32(vec) => vec.len(),
        }
    }

    pub fn upload(&self, buffer: &VulkanBuffer, memory: *mut c_void, size: vk::DeviceSize) {
        match self {
            Indices::U8(vec) => unsafe { buffer.upload_data(memory, vec, size) },
            Indices::U16(vec) => unsafe { buffer.upload_data(memory, vec, size) },
            Indices::U32(vec) => unsafe { buffer.upload_data(memory, vec, size) },
        }
    }

    pub fn get_slice(&self) -> &[u8] {
        match self {
            Indices::U8(indices) => cast_slice(&indices[..]),
            Indices::U16(indices) => cast_slice(&indices[..]),
            Indices::U32(indices) => cast_slice(&indices[..]),
        }
    }

    /// Returns `true` if there are no indices.
    pub fn is_empty(&self) -> bool {
        match self {
            Indices::U8(vec) => vec.is_empty(),
            Indices::U16(vec) => vec.is_empty(),
            Indices::U32(vec) => vec.is_empty(),
        }
    }

    pub fn vk_type(&self) -> vk::IndexType {
        match self {
            Indices::U8(_) => vk::IndexType::UINT8_KHR,
            Indices::U16(_) => vk::IndexType::UINT16,
            Indices::U32(_) => vk::IndexType::UINT32,
        }
    }
}

impl Vertex3D {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(mem::size_of::<Self>() as u32)
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

impl Vertex2D {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(mem::size_of::<Self>() as u32)
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
                .location(1)
                .binding(0)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(offset_of!(Self, color) as u32),
        ]
    }
}

pub fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) }
}

pub fn begin_single_time_command(
    device: &ash::Device,
    command_pool: vk::CommandPool,
) -> vk::CommandBuffer {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_buffer_count(1)
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffer = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    }[0];

    let command_buffer_begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Failed to begin recording Command Buffer at beginning!");
    }

    command_buffer
}

pub fn end_single_time_command(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    command_buffer: vk::CommandBuffer,
) {
    unsafe {
        device
            .end_command_buffer(command_buffer)
            .expect("Failed to record Command Buffer at Ending!");
    }

    let buffers_to_submit = vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer);

    let binding = [buffers_to_submit];
    let submit_info = vk::SubmitInfo2::default().command_buffer_infos(&binding);

    unsafe {
        let fence = device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .unwrap();

        device
            .queue_submit2(submit_queue, &[submit_info], fence)
            .expect("Failed to Queue Submit!");
        device
            .wait_for_fences(&[fence], true, DEFAULT_FENCE_TIMEOUT)
            .expect("Failed to wait for Fence");

        device.destroy_fence(fence, None);
        device.free_command_buffers(command_pool, &[command_buffer]);
    }
}
