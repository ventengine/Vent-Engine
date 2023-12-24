use std::{
    fs::{self, File},
    io::BufReader,
    mem::size_of,
    path::Path,
    sync, thread,
};

use ash::vk;
use vent_rendering::{
    buffer::VulkanBuffer, image::VulkanImage, instance::VulkanInstance, Vertex3D,
};

use crate::{Material, Model3D};

use super::{Mesh3D, ModelError};

pub(crate) struct GLTFLoader {}

impl GLTFLoader {
    pub async fn load(instance: &VulkanInstance, path: &Path) -> Result<Model3D, ModelError> {
        let doc = gltf::Gltf::from_reader(fs::File::open(path).unwrap()).unwrap();

        let path = path.parent().unwrap_or_else(|| Path::new("./"));

        let buffer_data = gltf::import_buffers(&doc, Some(path), doc.blob.clone())
            .expect("Failed to Load glTF Buffers");

        let mut meshes = Vec::new();
        doc.scenes().for_each(|scene| {
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
            Self::load_mesh_singlethreaded(instance, model_dir, mesh, buffer_data, meshes);
        }

        node.children()
            .for_each(|child| Self::load_node(instance, model_dir, child, buffer_data, meshes))
    }

    // fn load_mesh_multithreaded(
    //     instance: &VulkanInstance,
    //     camera_set: vk::WriteDescriptorSet,
    //     model_dir: &Path,
    //     mesh: gltf::Mesh,
    //     buffer_data: &[gltf::buffer::Data],
    //     meshes: &mut Vec<Mesh3D>,
    // ) {
    //     let primitive_len = mesh.primitives().size_hint().0;
    //     let (tx, rx) = sync::mpsc::sync_channel(primitive_len); // Create bounded channels

    //     // Spawn threads to load mesh primitive
    //     thread::scope(|s| {
    //         let tx = tx.clone();
    //         for primitive in mesh.primitives() {
    //             let tx = tx.clone();
    //             let camera_set = camera_set.clone();
    //             let mesh = mesh.clone();
    //             let instance = instance;
    //             let model_dir = model_dir;
    //             let buffer_data = buffer_data;

    //             s.spawn(move || {
    //                 let loaded_material =
    //                     Self::load_material(instance, camera_set, model_dir, primitive.material(), buffer_data);

    //                 let primitive = Self::load_primitive(buffer_data, primitive);

    //                 let loaded_mesh = Mesh3D::new(
    //                     &instance.device,
    //                     &instance.memory_allocator,
    //                     &primitive.0,
    //                     &primitive.1,
    //                     Some(loaded_material),
    //                     mesh.name(),
    //                 );

    //                 tx.send(loaded_mesh).unwrap();
    //             });
    //         }
    //     });
    //     for _ in 0..primitive_len {
    //         let mesh = rx.recv().unwrap();
    //         meshes.push(mesh);
    //     }
    // }

    #[deprecated]
    fn load_mesh_singlethreaded(
        instance: &VulkanInstance,
        model_dir: &Path,
        mesh: gltf::Mesh,
        buffer_data: &[gltf::buffer::Data],
        meshes: &mut Vec<Mesh3D>,
    ) {
        for primitive in mesh.primitives() {
            let loaded_material =
                Self::load_material(instance, model_dir, primitive.material(), buffer_data);

            let primitive = Self::load_primitive(buffer_data, primitive);

            let loaded_mesh = Mesh3D::new(
                &instance.device,
                &instance.memory_allocator,
                &primitive.0,
                &primitive.1,
                Some(loaded_material),
                mesh.name(),
            );
            meshes.push(loaded_mesh);
        }
    }

    fn load_material(
        instance: &VulkanInstance,
        model_dir: &Path,
        material: gltf::Material<'_>,
        buffer_data: &[gltf::buffer::Data],
        // image_data: &[gltf::image::Data],
    ) -> Material {
        let pbr = material.pbr_metallic_roughness();

        let diffuse_texture = if let Some(texture) = pbr.base_color_texture() {
            match texture.texture().source().source() {
                gltf::image::Source::View {
                    view,
                    mime_type: img_type,
                } => {
                    let image = image::load_from_memory_with_format(
                        &buffer_data[view.buffer().index()],
                        image::ImageFormat::from_mime_type(img_type).expect("TODO: Error Handling"),
                    )
                    .unwrap();
                    VulkanImage::from_image(
                        &instance.device,
                        image,
                        instance.command_pool,
                        &instance.memory_allocator,
                        instance.graphics_queue,
                        None,
                    )
                }
                gltf::image::Source::Uri { uri, mime_type } => {
                    let sampler = texture.texture().sampler();
                    let sampler = Self::convert_sampler(&sampler);
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

                    VulkanImage::from_image(
                        &instance.device,
                        image,
                        instance.command_pool,
                        &instance.memory_allocator,
                        instance.graphics_queue,
                        Some(sampler),
                    )
                }
            }
        } else {
            // Default Diffuse color
            VulkanImage::from_color(
                &instance.device,
                [255, 255, 255, 255],
                vk::Extent2D {
                    width: 128,
                    height: 128,
                },
            )
        };

        Material {
            diffuse_texture,
            base_color: pbr.base_color_factor(),
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
                gltf::texture::MinFilter::Linear
                | gltf::texture::MinFilter::LinearMipmapNearest => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::NearestMipmapNearest => {
                    (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
                }
                gltf::texture::MinFilter::LinearMipmapLinear => {
                    (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
                }
                _ => unimplemented!(),
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
