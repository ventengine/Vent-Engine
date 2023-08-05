use image::{GenericImageView, ImageError};
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
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn from_memory_to_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
    ) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, None, None, None, None, label)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        mag_filter: Option<wgpu::FilterMode>,
        min_filter: Option<wgpu::FilterMode>,
        mipmap_filter: Option<wgpu::FilterMode>,
        sampler_label: Option<&str>,
        texture_label: Option<&str>,
    ) -> Result<Self, ImageError> {
        Self::create(
            device,
            queue,
            &img.to_rgba8(),
            img.width(),
            img.height(),
            Self::DEFAULT_TEXTURE_FORMAT,
            mag_filter.unwrap_or(Self::DEFAULT_TEXTURE_FILTER),
            min_filter.unwrap_or(Self::DEFAULT_TEXTURE_FILTER),
            mipmap_filter.unwrap_or(Self::DEFAULT_TEXTURE_FILTER),
            sampler_label,
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
    ) -> Result<Self, ImageError> {
        let mut bytes = Vec::with_capacity((width * height) as usize);
        for _ in 0..height {
            for _ in 0..width {
                bytes.push(colors[0]);
                bytes.push(colors[1]);
                bytes.push(colors[2]);
                bytes.push(colors[3]);
            }
        }
        Self::create(
            device,
            queue,
            &bytes,
            width,
            height,
            Self::DEFAULT_TEXTURE_FORMAT,
            Self::DEFAULT_TEXTURE_FILTER,
            Self::DEFAULT_TEXTURE_FILTER,
            Self::DEFAULT_TEXTURE_FILTER,
            None,
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
        mag_filter: wgpu::FilterMode,
        min_filter: wgpu::FilterMode,
        mipmap_filter: wgpu::FilterMode,
        sampler_label: Option<&str>,
        texture_label: Option<&str>,
    ) -> Result<Self, ImageError> {
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
                view_formats: &[],
            },
            bytes,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: sampler_label,
            mag_filter,
            min_filter,
            mipmap_filter,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
