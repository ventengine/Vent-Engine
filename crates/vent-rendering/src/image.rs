use ash::vk::{self, Extent2D};

pub struct VulkanImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl VulkanImage {
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

    pub fn new_depth(device: &ash::Device, format: vk::Format, size: Extent2D) -> Self {
        let image = Self::create_image(
            device,
            format,
            size,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        let image_view =
            Self::create_image_view(image, device, format, vk::ImageAspectFlags::DEPTH);

        let sampler = Self::create_sampler(device);

        Self {
            image,
            image_view,
            sampler,
        }
    }

    pub fn new_from_color(device: &ash::Device, color: [u8; 4], size: Extent2D) {}

    fn copy_buffer_to_image(
        device: &ash::Device,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    )  {
        let command_buffer = begin_single_time_commands(device)?;
    
        let subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);
    
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
            });
    
        device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    
        end_single_time_commands(device, command_buffer)?;
    }

    fn create_sampler(device: &ash::Device) -> vk::Sampler {
        let create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
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
            .build();
        unsafe { device.create_sampler(&create_info, None) }.unwrap()
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
