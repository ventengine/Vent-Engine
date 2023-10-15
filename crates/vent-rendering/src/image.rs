use ash::vk::{self, Extent2D};

use crate::{
    allocator::MemoryAllocator, begin_single_time_command, buffer::VulkanBuffer,
    end_single_time_command,
};

pub struct VulkanImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl VulkanImage {
    pub const DEFAULT_TEXTURE_FILTER: vk::Filter = vk::Filter::LINEAR;

    pub fn new(
        device: &ash::Device,
        format: vk::Format,
        sampler_info: vk::SamplerCreateInfo,
        size: Extent2D,
        usage: vk::ImageUsageFlags,
    ) -> Self {
        let image = Self::create_image(device, format, size, usage);
        let image_view =
            Self::create_image_view(image, device, format, vk::ImageAspectFlags::COLOR);

        let sampler = unsafe { device.create_sampler(&sampler_info, None) }.unwrap();

        Self {
            image,
            image_view,
            sampler,
        }
    }

    pub fn from_image(
        device: &ash::Device,
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
                image.to_rgba8().into_raw()
            }
            image::DynamicImage::ImageLumaA8(_) | image::DynamicImage::ImageRgba8(_) => {
                image.into_bytes()
            }
            _ => image.to_rgb8().into_raw(),
        };
        let image_data_size =
            (std::mem::size_of::<u8>() as u32 * image_size.width * image_size.height * 4)
                as vk::DeviceSize;
        let vk_image = Self::new(
            device,
            vk::Format::R8G8B8A8_UNORM,
            sampler_info.unwrap_or(Self::default_sampler()),
            image_size,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        );
        let buffer = VulkanBuffer::new_init(
            device,
            allocator,
            image_data_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            &image_data,
        );
        vk_image.copy_buffer_to_image(
            device,
            command_pool,
            submit_queue,
            *buffer,
            image_size.width,
            image_size.height,
        );
        vk_image
    }

    pub fn new_depth(device: &ash::Device, format: vk::Format, size: Extent2D) -> Self {
        let image = Self::create_image(
            device,
            format,
            size,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let image_view =
            Self::create_image_view(image, device, format, vk::ImageAspectFlags::DEPTH);

        let sampler = unsafe { device.create_sampler(&Self::default_sampler(), None) }.unwrap();

        Self {
            image,
            image_view,
            sampler,
        }
    }

    pub fn from_color(_device: &ash::Device, color: [u8; 4], size: Extent2D) -> Self {
        let _img = image::RgbaImage::from_pixel(size.width, size.height, image::Rgba(color));
        todo!()
    }

    pub fn copy_buffer_to_image(
        &self,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        buffer: vk::Buffer,
        width: u32,
        height: u32,
    ) {
        let command_buffer = begin_single_time_command(device, command_pool);

        let subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let region = vk::BufferImageCopy::builder()
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

        unsafe {
            device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            )
        };

        end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    fn default_sampler() -> vk::SamplerCreateInfo {
        vk::SamplerCreateInfo::builder()
            .mag_filter(Self::DEFAULT_TEXTURE_FILTER)
            .min_filter(Self::DEFAULT_TEXTURE_FILTER)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(1.0)
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
        let depth_image_view_info = vk::ImageViewCreateInfo::builder()
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

        unsafe { device.create_image_view(&depth_image_view_info, None) }.unwrap()
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

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.destroy_sampler(self.sampler, None);
        }
    }
}
