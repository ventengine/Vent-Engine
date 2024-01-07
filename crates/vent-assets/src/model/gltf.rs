use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
    sync::{self},
    thread,
};

use ash::vk;
use gltf::texture::Sampler;
use image::DynamicImage;
use vent_rendering::{image::VulkanImage, instance::VulkanInstance, Vertex3D};

use crate::{Material, Model3D};

use super::{Mesh3D, ModelError};

pub(crate) struct GLTFLoader {}

struct MaterialData<'a> {
    image: DynamicImage,
    sampler: Option<Sampler<'a>>,
    base_color: [f32; 4],
}

impl GLTFLoader {
    pub async fn load(instance: &VulkanInstance, path: &Path) -> Result<Model3D, ModelError> {
        let gltf = gltf::Gltf::from_reader(fs::File::open(path).unwrap()).unwrap();

        let path = path.parent().unwrap_or_else(|| Path::new("./"));

        let buffer_data = gltf::import_buffers(&gltf.document, Some(path), gltf.blob)
            .expect("Failed to Load glTF Buffers");

        let mut meshes = Vec::new();
        gltf.document.scenes().for_each(|scene| {
            scene.nodes().for_each(|node| {
                Self::load_node(instance, path, node, &buffer_data, &mut meshes);
            })
        });

        Ok(Model3D { meshes })
    }

    fn load_node(
        instance: &VulkanInstance,
        model_dir: &Path,
        node: gltf::Node<'_>,
        buffer_data: &[gltf::buffer::Data],
        meshes: &mut Vec<Mesh3D>,
    ) {
        if let Some(mesh) = node.mesh() {
            Self::load_mesh_multithreaded(instance, model_dir, mesh, buffer_data, meshes);
        }

        node.children()
            .for_each(|child| Self::load_node(instance, model_dir, child, buffer_data, meshes))
    }

    fn load_mesh_multithreaded(
        instance: &VulkanInstance,
        model_dir: &Path,
        mesh: gltf::Mesh,
        buffer_data: &[gltf::buffer::Data],
        meshes: &mut Vec<Mesh3D>,
    ) {
        let primitive_len = mesh.primitives().size_hint().0;
        let (tx, rx) = sync::mpsc::sync_channel(primitive_len); // Create bounded channels

        // Spawn threads to load mesh
        thread::scope(|s| {
            for primitive in mesh.primitives() {
                let tx = tx.clone();

                s.spawn(move || {
                    let material_data =
                        Self::parse_material_data(model_dir, primitive.material(), buffer_data);

                    let primitive = Self::load_primitive(buffer_data, primitive);

                    tx.send((material_data, primitive)).unwrap();
                });
            }
        });
        for _ in 0..primitive_len {
            let (material_data, primitive) = rx.recv().unwrap();
            let material = Self::load_material(instance, material_data);
            let loaded_mesh = Mesh3D::new(
                instance,
                &instance.memory_allocator,
                &primitive.0,
                &primitive.1,
                Some(material),
                mesh.name(),
            );
            meshes.push(loaded_mesh);
        }
    }

    /**
     *  We will parse all Materials and save that what we need
     *  the Data will be saved on RAM
     */
    fn parse_material_data<'a>(
        model_dir: &'a Path,
        material: gltf::Material<'a>,
        buffer_data: &'a [gltf::buffer::Data],
        // image_data: &[gltf::image::Data],
    ) -> MaterialData<'a> {
        let pbr = material.pbr_metallic_roughness();

        let diffuse_texture = if let Some(texture) = pbr.base_color_texture() {
            match texture.texture().source().source() {
                gltf::image::Source::View { view, mime_type } => {
                    let sampler = texture.texture().sampler();
                    let image = image::load_from_memory_with_format(
                        &buffer_data[view.buffer().index()],
                        image::ImageFormat::from_mime_type(mime_type)
                            .expect("TODO: Error Handling"),
                    )
                    .unwrap();
                    (image, Some(sampler))
                }
                gltf::image::Source::Uri { uri, mime_type } => {
                    let sampler = texture.texture().sampler();
                    let image = if let Some(mime_type) = mime_type {
                        image::load(
                            BufReader::new(File::open(model_dir.join(uri)).unwrap()),
                            image::ImageFormat::from_mime_type(mime_type)
                                .expect("TODO: Error Handling"),
                        )
                        .unwrap()
                    } else {
                        image::open(model_dir.join(uri)).unwrap()
                    };

                    (image, Some(sampler))
                }
            }
        } else {
            (
                image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
                    128,
                    128,
                    image::Rgba([255, 255, 255, 255]),
                )),
                None,
            )
        };
        MaterialData {
            image: diffuse_texture.0,
            sampler: diffuse_texture.1,
            base_color: pbr.base_color_factor(),
        }
    }

    /**
     *  Creates an VulkanImage from Material Data, We want to do this Single threaded
     *  RAM -> VRAM
     */
    fn load_material(instance: &VulkanInstance, data: MaterialData) -> Material {
        let diffuse_texture = VulkanImage::from_image(
            instance,
            data.image,
            instance.command_pool,
            &instance.memory_allocator,
            instance.graphics_queue,
            data.sampler.map(|s| Self::convert_sampler(&s)),
        );

        Material {
            diffuse_texture,
            base_color: data.base_color,
        }
    }

    /// Converts an gltf Texture Sampler into Vulkan Sampler Info
    #[must_use]
    fn convert_sampler(sampler: &gltf::texture::Sampler) -> vk::SamplerCreateInfo {
        let mag_filter = sampler.mag_filter().map_or(
            VulkanImage::DEFAULT_TEXTURE_FILTER,
            |filter| match filter {
                gltf::texture::MagFilter::Nearest => vk::Filter::NEAREST,
                gltf::texture::MagFilter::Linear => vk::Filter::LINEAR,
            },
        );

        let (min_filter, mipmap_filter) = sampler.min_filter().map_or(
            (
                VulkanImage::DEFAULT_TEXTURE_FILTER,
                vk::SamplerMipmapMode::LINEAR,
            ),
            |filter| match filter {
                gltf::texture::MinFilter::Nearest => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::Linear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::NearestMipmapNearest => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::LinearMipmapNearest => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::NearestMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
                gltf::texture::MinFilter::LinearMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
            },
        );

        let address_mode_u = Self::conv_wrapping_mode(sampler.wrap_s());
        let address_mode_v = Self::conv_wrapping_mode(sampler.wrap_t());

        vk::SamplerCreateInfo::builder()
            .mag_filter(mag_filter)
            .min_filter(min_filter)
            .mipmap_mode(mipmap_filter)
            .address_mode_u(address_mode_u)
            .address_mode_v(address_mode_v)
            .build()
    }

    #[must_use]
    const fn conv_wrapping_mode(mode: gltf::texture::WrappingMode) -> vk::SamplerAddressMode {
        match mode {
            gltf::texture::WrappingMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
            gltf::texture::WrappingMode::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
            gltf::texture::WrappingMode::Repeat => vk::SamplerAddressMode::REPEAT,
        }
    }

    fn load_primitive(
        buffer_data: &[gltf::buffer::Data],
        primitive: gltf::Primitive,
    ) -> (Vec<Vertex3D>, Vec<u32>) {
        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

        let mut vertices: Vec<Vertex3D> = reader
            .read_positions()
            .unwrap()
            .map(|position| Vertex3D {
                position,
                tex_coord: Default::default(),
                normal: Default::default(),
            })
            .collect();

        if let Some(normal_attribute) = reader.read_normals() {
            for (normal_index, normal) in normal_attribute.enumerate() {
                vertices[normal_index].normal = normal;
            }
        }

        if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
            for (tex_coord_index, tex_coord) in tex_coord_attribute.enumerate() {
                vertices[tex_coord_index].tex_coord = tex_coord;
            }
        }

        let indices: Vec<_> = reader.read_indices().unwrap().into_u32().collect();
        (vertices, indices)
    }
}
