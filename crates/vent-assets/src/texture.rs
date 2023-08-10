use image::{ImageError, GenericImageView};
use wgpu::util::DeviceExt;

use crate::Texture;

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const DEFAULT_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const DEFAULT_TEXTURE_FILTER: wgpu::FilterMode = wgpu::FilterMode::Linear;

    #[must_use]
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: Option<&str>,
    ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[Self::DEPTH_FORMAT],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    // This Should be Prefered
    pub fn from_memory_to_image_with_format(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        format: image::ImageFormat,
        label: Option<&str>,
    ) -> Result<Self, ImageError> {
        let img = image::load_from_memory_with_format(bytes, format)?;
        Ok(Self::from_image(device, queue, img, None, label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: image::DynamicImage,
        sampler_desc: Option<&wgpu::SamplerDescriptor>,
        texture_label: Option<&str>,
    ) -> Self {
        let dimensions = img.dimensions();
        Self::create(
            device,
            queue,
            &img.into_rgba8(),
           dimensions.0,
           dimensions.1,
            Self::DEFAULT_TEXTURE_FORMAT,
            sampler_desc.unwrap_or(&wgpu::SamplerDescriptor::default()),
            texture_label,
        )
    }

    pub fn from_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        colors: [u8; 4],
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Self {
        let mut bytes = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..height {
            for _ in 0..width {
                bytes.extend_from_slice(&colors);
            }
        }
        Self::create(
            device,
            queue,
            &bytes,
            width,
            height,
            Self::DEFAULT_TEXTURE_FORMAT,
            &wgpu::SamplerDescriptor {
                mag_filter: Self::DEFAULT_TEXTURE_FILTER,
                min_filter: Self::DEFAULT_TEXTURE_FILTER,
                mipmap_filter: Self::DEFAULT_TEXTURE_FILTER,
                ..Default::default()
            },
            label,
        )
    }

    pub fn create(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        sampler_desc: &wgpu::SamplerDescriptor,
        texture_label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: texture_label,
                view_formats: &[format],
            },
            bytes,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(sampler_desc);

        Self {
            texture,
            view,
            sampler,
        }
    }
}
