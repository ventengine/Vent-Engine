use std::{fs, io, path::Path};

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
        let full_model =
            gltf::Gltf::from_reader(io::BufReader::new(fs::File::open(path).unwrap())).unwrap();
        let path = path.parent().unwrap_or_else(|| Path::new("./"));

        // TODO: Error Handling
        let buffer_data = gltf::import_buffers(&full_model, Some(path), full_model.blob.clone())
            .expect("Failed to Load Buffers :C");

        let mut meshes = Vec::new();
        full_model.scenes().for_each(|scene| {
            scene.nodes().for_each(|node| {
                Self::load_node(device, node, &buffer_data, &mut meshes);
            })
        });

        let mut materials = Vec::with_capacity(full_model.materials().len());
        for material in full_model.materials() {
            materials.push(
                Self::load_material(
                    device,
                    queue,
                    path,
                    material,
                    &buffer_data,
                    texture_bind_group_layout,
                )
                .await,
            );
        }

        Ok(Model3D { meshes, materials })
    }

    fn load_node(
        device: &wgpu::Device,
        node: gltf::Node<'_>,
        buffer_data: &[gltf::buffer::Data],
        meshes: &mut Vec<Mesh3D>,
    ) {
        if let Some(mesh) = node.mesh() {
            mesh.primitives().for_each(|primitive| {
                meshes.push(Self::load_primitive(
                    device,
                    mesh.name(),
                    buffer_data,
                    primitive,
                ));
            })
        }

        node.children()
            .for_each(|child| Self::load_node(device, child, buffer_data, meshes))
    }

    async fn load_material(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model_dir: &Path,
        material: gltf::Material<'_>,
        buffer_data: &[gltf::buffer::Data],
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
                gltf::image::Source::Uri { uri, mime_type: _ } => {
                    let sampler = texture.texture().sampler();
                    let wgpu_sampler = Self::convert_sampler(&sampler);
                    Texture::from_image(
                        device,
                        queue,
                        image::open(model_dir.join(uri)).unwrap(),
                        Some(&wgpu_sampler),
                        texture.texture().name(),
                    )
                }
            }
        } else {
            Texture::from_color(device, queue, [255, 255, 255, 255], 128, 128, None)
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
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
    fn convert_sampler<'a>(sampler: &'a gltf::texture::Sampler<'a>) -> wgpu::SamplerDescriptor<'a> {
        let mag_filter = if let Some(filter) = sampler.mag_filter() {
            match filter {
                gltf::texture::MagFilter::Nearest => wgpu::FilterMode::Nearest,
                gltf::texture::MagFilter::Linear => wgpu::FilterMode::Linear,
            }
        } else {
            Texture::DEFAULT_TEXTURE_FILTER
        };
        let (min_filter, mipmap_filter) = if let Some(filter) = sampler.min_filter() {
            let mut min = Texture::DEFAULT_TEXTURE_FILTER;
            let mut mipmap = Texture::DEFAULT_TEXTURE_FILTER;
            match filter {
                gltf::texture::MinFilter::Nearest => min = wgpu::FilterMode::Nearest,
                gltf::texture::MinFilter::Linear
                | gltf::texture::MinFilter::LinearMipmapNearest => min = wgpu::FilterMode::Linear,
                gltf::texture::MinFilter::NearestMipmapNearest => {
                    mipmap = wgpu::FilterMode::Nearest
                }

                gltf::texture::MinFilter::LinearMipmapLinear => mipmap = wgpu::FilterMode::Linear,
                _ => unimplemented!(),
            }
            (min, mipmap)
        } else {
            (
                Texture::DEFAULT_TEXTURE_FILTER,
                Texture::DEFAULT_TEXTURE_FILTER,
            )
        };
        let address_mode_u = Self::conv_wrapping_mode(&sampler.wrap_s());
        let address_mode_v = Self::conv_wrapping_mode(&sampler.wrap_t());
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

    fn conv_wrapping_mode(mode: &gltf::texture::WrappingMode) -> wgpu::AddressMode {
        match mode {
            gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        }
    }

    fn load_primitive(
        device: &wgpu::Device,
        name: Option<&str>,
        buffer_data: &[gltf::buffer::Data],
        primitive: gltf::Primitive,
    ) -> Mesh3D {
        let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

        let mut vertices = Vec::new();
        if let Some(vertex_attribute) = reader.read_positions() {
            vertices.reserve(vertex_attribute.len());
            vertex_attribute.for_each(|vertex| {
                vertices.push(Vertex3D {
                    position: vertex,
                    tex_coord: Default::default(),
                    normal: Default::default(),
                })
            });
        }
        if let Some(normal_attribute) = reader.read_normals() {
            let mut normal_index = 0;
            normal_attribute.for_each(|normal| {
                vertices[normal_index].normal = normal;

                normal_index += 1;
            });
        }
        if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
            let mut tex_coord_index = 0;
            tex_coord_attribute.for_each(|tex_coord| {
                vertices[tex_coord_index].tex_coord = tex_coord;

                tex_coord_index += 1;
            });
        }

        let indices: Vec<_> = reader.read_indices().unwrap().into_u32().collect();

        Mesh3D::new(
            device,
            &vertices,
            &indices,
            primitive.material().index().unwrap_or(0),
            name,
        )
    }
}
