use std::{
    fs::{self, File},
    io::BufReader,
    path::Path,
    sync, thread,
};

use wgpu::{util::DeviceExt, BindGroupLayout};

use crate::{Model3D, Texture, Vertex3D};

use super::{Material, Mesh3D, ModelError};

pub(crate) struct GLTFLoader {}

impl GLTFLoader {
    pub async fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Result<Model3D, ModelError> {
        let doc = gltf::Gltf::from_reader(fs::File::open(path).unwrap()).unwrap();

        let path = path.parent().unwrap_or_else(|| Path::new("./"));

        let buffer_data = gltf::import_buffers(&doc, Some(path), doc.blob.clone())
            .expect("Failed to Load glTF Buffers");

        let mut meshes = Vec::new();
        doc.scenes().for_each(|scene| {
            scene.nodes().for_each(|node| {
                Self::load_node(
                    device,
                    queue,
                    path,
                    node,
                    &buffer_data,
                    texture_bind_group_layout,
                    &mut meshes,
                );
            })
        });

        Ok(Model3D { meshes })
    }

    fn load_node(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model_dir: &Path,
        node: gltf::Node<'_>,
        buffer_data: &[gltf::buffer::Data],
        texture_bind_group_layout: &BindGroupLayout,
        meshes: &mut Vec<Mesh3D>,
    ) {
        if let Some(mesh) = node.mesh() {
            Self::load_mesh_multithreaded(
                device,
                queue,
                model_dir,
                mesh,
                buffer_data,
                texture_bind_group_layout,
                meshes,
            );
        }

        node.children().for_each(|child| {
            Self::load_node(
                device,
                queue,
                model_dir,
                child,
                buffer_data,
                texture_bind_group_layout,
                meshes,
            )
        })
    }

    fn load_mesh_multithreaded(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model_dir: &Path,
        mesh: gltf::Mesh,
        buffer_data: &[gltf::buffer::Data],
        texture_bind_group_layout: &BindGroupLayout,
        meshes: &mut Vec<Mesh3D>,
    ) {
        let primitive_len = mesh.primitives().size_hint().0;
        let (tx, rx) = sync::mpsc::sync_channel(primitive_len); // Create bounded channels

        // Spawn threads to load mesh primitive
        thread::scope(|s| {
            let tx = tx.clone();
            for primitive in mesh.primitives() {
                let tx = tx.clone();
                let mesh = mesh.clone();
                let device = device;
                let queue = queue;
                let model_dir = model_dir;
                let buffer_data = buffer_data;
                let texture_bind_group_layout = texture_bind_group_layout;

                s.spawn(move || {
                    let loaded_material = Self::load_material(
                        device,
                        queue,
                        model_dir,
                        primitive.material(),
                        buffer_data,
                        texture_bind_group_layout,
                    );

                    let loaded_mesh = Self::load_primitive(
                        device,
                        loaded_material,
                        mesh.name(),
                        buffer_data,
                        primitive,
                    );
                    tx.send(loaded_mesh).unwrap();
                });
            }
        });
        for _ in 0..primitive_len {
            let mesh = rx.recv().unwrap();
            meshes.push(mesh);
        }
    }

    fn load_material(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model_dir: &Path,
        material: gltf::Material<'_>,
        buffer_data: &[gltf::buffer::Data],
        // image_data: &[gltf::image::Data],
        texture_bind_group_layout: &BindGroupLayout,
    ) -> wgpu::BindGroup {
        let pbr = material.pbr_metallic_roughness();

        let diffuse_texture = if let Some(texture) = pbr.base_color_texture() {
            match texture.texture().source().source() {
                gltf::image::Source::View {
                    view,
                    mime_type: img_type,
                } => Texture::from_memory_to_image_with_format(
                    device,
                    queue,
                    &buffer_data[view.buffer().index()],
                    image::ImageFormat::from_mime_type(img_type).expect("TODO: Error Handling"),
                    texture.texture().name(),
                )
                .unwrap(),
                gltf::image::Source::Uri { uri, mime_type } => {
                    let sampler = texture.texture().sampler();
                    let wgpu_sampler = Self::convert_sampler(&sampler);
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

                    Texture::from_image(
                        device,
                        queue,
                        image,
                        Some(&wgpu_sampler),
                        texture.texture().name(),
                    )
                }
            }
        } else {
            Texture::from_color(device, queue, [255, 255, 255, 255], 128, 128, None)
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Texture Uniform Buffer"),
            contents: bytemuck::bytes_of(&Material {
                base_color: pbr.base_color_factor(),
            }),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer.as_entire_binding(),
                },
            ],
            label: material.name(),
        })
    }

    /// Converts an gltf Texture Sampler into WGPU Filter Modes
    #[must_use]
    fn convert_sampler<'a>(sampler: &'a gltf::texture::Sampler<'a>) -> wgpu::SamplerDescriptor<'a> {
        let mag_filter = sampler
            .mag_filter()
            .map_or(Texture::DEFAULT_TEXTURE_FILTER, |filter| match filter {
                gltf::texture::MagFilter::Nearest => wgpu::FilterMode::Nearest,
                gltf::texture::MagFilter::Linear => wgpu::FilterMode::Linear,
            });

        let (min_filter, mipmap_filter) = sampler.min_filter().map_or(
            (
                Texture::DEFAULT_TEXTURE_FILTER,
                Texture::DEFAULT_TEXTURE_FILTER,
            ),
            |filter| match filter {
                gltf::texture::MinFilter::Nearest => {
                    (wgpu::FilterMode::Nearest, Texture::DEFAULT_TEXTURE_FILTER)
                }
                gltf::texture::MinFilter::Linear
                | gltf::texture::MinFilter::LinearMipmapNearest => {
                    (wgpu::FilterMode::Linear, Texture::DEFAULT_TEXTURE_FILTER)
                }
                gltf::texture::MinFilter::NearestMipmapNearest => {
                    (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest)
                }
                gltf::texture::MinFilter::LinearMipmapLinear => {
                    (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear)
                }
                _ => unimplemented!(),
            },
        );

        let address_mode_u = Self::conv_wrapping_mode(sampler.wrap_s());
        let address_mode_v = Self::conv_wrapping_mode(sampler.wrap_t());

        wgpu::SamplerDescriptor {
            label: sampler.name(),
            mag_filter,
            min_filter,
            mipmap_filter,
            address_mode_u,
            address_mode_v,
            ..Default::default()
        }
    }

    #[must_use]
    fn conv_wrapping_mode(mode: gltf::texture::WrappingMode) -> wgpu::AddressMode {
        match mode {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        }
    }

    fn load_primitive(
        device: &wgpu::Device,
        bind_group: wgpu::BindGroup,
        name: Option<&str>,
        buffer_data: &[gltf::buffer::Data],
        primitive: gltf::Primitive,
    ) -> Mesh3D {
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

        Mesh3D::new(device, &vertices, &indices, Some(bind_group), name)
    }
}
