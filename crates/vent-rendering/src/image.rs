use ash::vk::{self, Extent2D};

use crate::{
    allocator::MemoryAllocator, begin_single_time_command, buffer::VulkanBuffer,
    end_single_time_command, instance::VulkanInstance,
};

pub struct DepthImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub memory: vk::DeviceMemory,
}

impl DepthImage {
    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);
        }
    }
}

pub struct VulkanImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub memory: Option<vk::DeviceMemory>,
}

impl VulkanImage {
    pub const DEFAULT_TEXTURE_FILTER: vk::Filter = vk::Filter::LINEAR;

    pub fn from_image(
        instance: &VulkanInstance,
        image: image::DynamicImage,
        command_pool: vk::CommandPool,
        allocator: &MemoryAllocator,
        submit_queue: vk::Queue,
        sampler_info: Option<vk::SamplerCreateInfo>,
    ) -> Self {
        let image_size = Extent2D {
            width: image.width(),
            height: image.height(),
        };
        let image_data = match &image {
            image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageRgb8(_) => {
                image.into_rgba8().into_raw()
            }
            image::DynamicImage::ImageLumaA8(_) | image::DynamicImage::ImageRgba8(_) => {
                image.into_bytes()
            }
            _ => image.into_rgb8().into_raw(),
        };
        let image_data_size = (image_size.width * image_size.height * 4) as vk::DeviceSize;

        let mut staging_buffer = VulkanBuffer::new_init(
            instance,
            allocator,
            image_data_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            &image_data,
            None,
        );

        let image = Self::create_image(
            &instance.device,
            vk::Format::R8G8B8A8_UNORM,
            image_size,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        );
        let memory = VulkanBuffer::new_image(&instance.device, allocator, image);

        Self::copy_buffer_to_image(
            &instance.device,
            image,
            &staging_buffer,
            command_pool,
            submit_queue,
            image_size.width,
            image_size.height,
        );
        staging_buffer.destroy(&instance.device);

        let image_view = Self::create_image_view(
            image,
            &instance.device,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageAspectFlags::COLOR,
        );

        let sampler = unsafe {
            instance
                .device
                .create_sampler(&sampler_info.unwrap_or(Self::default_sampler()), None)
        }
        .unwrap();

        Self {
            image,
            image_view,
            sampler,
            memory: Some(memory),
        }
    }

    pub fn new_depth(
        device: &ash::Device,
        allocator: &MemoryAllocator,
        format: vk::Format,
        size: Extent2D,
    ) -> DepthImage {
        let image = Self::create_image(
            device,
            format,
            size,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let memory = VulkanBuffer::new_image(device, allocator, image);
        let image_view =
            Self::create_image_view(image, device, format, vk::ImageAspectFlags::DEPTH);

        DepthImage {
            image,
            image_view,
            memory,
        }
    }

    pub fn from_color(
        instance: &VulkanInstance,
        command_pool: vk::CommandPool,
        allocator: &MemoryAllocator,
        submit_queue: vk::Queue,
        color: [u8; 4],
        size: Extent2D,
    ) -> Self {
        let color_img = image::RgbaImage::from_pixel(size.width, size.height, image::Rgba(color));
        Self::from_image(
            instance,
            image::DynamicImage::ImageRgba8(color_img),
            command_pool,
            allocator,
            submit_queue,
            None,
        )
    }

    pub fn copy_buffer_to_image(
        device: &ash::Device,
        image: vk::Image,
        staging_buffer: &VulkanBuffer,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        width: u32,
        height: u32,
    ) {
        let command_buffer = begin_single_time_command(device, command_pool);

        let image_barrier = vk::ImageMemoryBarrier2::builder()
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
            .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            })
            .build();

        let dep_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&[image_barrier])
            .dependency_flags(vk::DependencyFlags::BY_REGION)
            .build();

        unsafe { device.cmd_pipeline_barrier2(command_buffer, &dep_info) };

        let subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let region = vk::BufferImageCopy2::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(subresource)
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .build();

        let copy_image_info = vk::CopyBufferToImageInfo2::builder()
            .src_buffer(staging_buffer.buffer)
            .dst_image(image)
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .regions(&[region])
            .build();

        unsafe { device.cmd_copy_buffer_to_image2(command_buffer, &copy_image_info) };

        let image_barrier = vk::ImageMemoryBarrier2::builder()
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags2::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            })
            .build();

        let dep_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&[image_barrier])
            .dependency_flags(vk::DependencyFlags::BY_REGION)
            .build();

        unsafe { device.cmd_pipeline_barrier2(command_buffer, &dep_info) };

        end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    pub fn default_sampler() -> vk::SamplerCreateInfo {
        vk::SamplerCreateInfo::builder()
            .mag_filter(Self::DEFAULT_TEXTURE_FILTER)
            .min_filter(Self::DEFAULT_TEXTURE_FILTER)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .build()
    }

    fn create_image_view(
        image: vk::Image,
        device: &ash::Device,
        format: vk::Format,
        mask: vk::ImageAspectFlags,
    ) -> vk::ImageView {
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(mask)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .build();

        unsafe { device.create_image_view(&image_view_info, None) }.unwrap()
    }

    fn create_image(
        device: &ash::Device,
        format: vk::Format,
        size: Extent2D,
        usage: vk::ImageUsageFlags,
    ) -> vk::Image {
        let create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(size.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        unsafe { device.create_image(&create_info, None) }.unwrap()
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.destroy_sampler(self.sampler, None);
            if let Some(memory) = self.memory {
                device.free_memory(memory, None);
            }
        }
    }
}
