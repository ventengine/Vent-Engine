use std::{
    os::raw::c_void,
};

use ash::vk;
use buffer::VulkanBuffer;
use bytemuck::cast_slice;
use ordered_float::OrderedFloat;

pub mod allocator;
pub mod buffer;
mod debug;
pub mod image;
pub mod instance;
pub mod mesh;
pub mod pipeline;
mod surface;
pub mod vertex;

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

pub enum Indices {
    U8(Vec<u8>), // TODO: Enable uin8 vulkan feature
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

    /// # Safety
    ///
    /// Do not give an bad memory pointer
    pub unsafe fn upload(&self, buffer: &VulkanBuffer, memory: *mut c_void, size: vk::DeviceSize) {
        match self {
            Indices::U8(vec) => buffer.upload_data(memory, vec, size),
            Indices::U16(vec) => buffer.upload_data(memory, vec, size),
            Indices::U32(vec) => buffer.upload_data(memory, vec, size),
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
