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
    pub memory: vk::DeviceMemory,
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
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            None,
        );

        let mip_level = (image_size.width.max(image_size.height) as f32)
            .log2()
            .floor() as u32
            + 1;

        let format = vk::Format::R8G8B8A8_UNORM;

        let image = Self::create_image(
            &instance.device,
            format,
            image_size,
            mip_level,
            vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::SAMPLED,
        );
        let memory = VulkanBuffer::new_image(&instance.device, allocator, image);

        Self::copy_buffer_to_image(
            &instance.device,
            image,
            &staging_buffer,
            command_pool,
            submit_queue,
            image_size,
            mip_level,
        );
        staging_buffer.destroy(&instance.device);

        Self::generate_mipmaps(
            &instance.device,
            image,
            command_pool,
            submit_queue,
            image_size.width,
            image_size.height,
            mip_level,
        );

        let image_view = Self::create_image_view(
            image,
            &instance.device,
            format,
            mip_level,
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
            memory,
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
            1,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let memory = VulkanBuffer::new_image(device, allocator, image);
        let image_view =
            Self::create_image_view(image, device, format, 1, vk::ImageAspectFlags::DEPTH);

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
        size: Extent2D,
        mip_level: u32,
    ) {
        let command_buffer = begin_single_time_command(device, command_pool);

        let image_barrier = vk::ImageMemoryBarrier2::builder()
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(image)
            .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
            .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: mip_level,
                layer_count: 1,
                ..Default::default()
            });

        let binding = [*image_barrier];
        let dep_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&binding)
            .dependency_flags(vk::DependencyFlags::BY_REGION);

        unsafe { device.cmd_pipeline_barrier2(command_buffer, &dep_info) };

        let subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let region = vk::BufferImageCopy2::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(*subresource)
            .image_offset(vk::Offset3D::default())
            .image_extent(size.into());

        let binding = [*region];
        let copy_image_info = vk::CopyBufferToImageInfo2::builder()
            .src_buffer(staging_buffer.buffer)
            .dst_image(image)
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .regions(&binding);

        unsafe { device.cmd_copy_buffer_to_image2(command_buffer, &copy_image_info) };

        end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    pub fn generate_mipmaps(
        device: &ash::Device,
        image: vk::Image,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        width: u32,
        height: u32,
        mip_level: u32,
    ) {
        let command_buffer = begin_single_time_command(device, command_pool);

        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);

        let mut barrier = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(*subresource);

        let mut mip_width = width as i32;
        let mut mip_height = height as i32;

        for i in 1..mip_level {
            barrier.subresource_range.base_mip_level = i - 1;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[*barrier],
                )
            };

            let src_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i - 1)
                .base_array_layer(0)
                .layer_count(1);

            let dst_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i)
                .base_array_layer(0)
                .layer_count(1);

            let blit = vk::ImageBlit::builder()
                .src_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ])
                .src_subresource(*src_subresource)
                .dst_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: (if mip_width > 1 { mip_width / 2 } else { 1 }),
                        y: (if mip_height > 1 { mip_height / 2 } else { 1 }),
                        z: 1,
                    },
                ])
                .dst_subresource(*dst_subresource);

            unsafe {
                device.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[*blit],
                    vk::Filter::LINEAR,
                )
            };

            barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[*barrier],
                )
            };

            if mip_width > 1 {
                mip_width /= 2;
            }

            if mip_height > 1 {
                mip_height /= 2;
            }
        }

        barrier.subresource_range.base_mip_level = mip_level - 1;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*barrier],
            )
        };

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
        mip_level: u32,
        mask: vk::ImageAspectFlags,
    ) -> vk::ImageView {
        let image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(mask)
                    .level_count(mip_level)
                    .layer_count(1)
                    .build(),
            )
            .image(image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D);

        unsafe { device.create_image_view(&image_view_info, None) }.unwrap()
    }

    fn create_image(
        device: &ash::Device,
        format: vk::Format,
        size: Extent2D,
        mip_level: u32,
        usage: vk::ImageUsageFlags,
    ) -> vk::Image {
        let create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(size.into())
            .mip_levels(mip_level)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        unsafe { device.create_image(&create_info, None) }.unwrap()
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.destroy_sampler(self.sampler, None);
            device.free_memory(self.memory, None);
        }
    }
}
