use std::{path::Path};

use glam::{Quat, Vec3};
use wgpu::BindGroupLayout;

use crate::{Model3D, Texture, Vertex3D};

use super::{Mesh3D, ModelError};

pub(crate) struct GLTFLoader {}

impl GLTFLoader {
    pub async fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Result<Model3D, ModelError> {
        let (gltf, buffers, _images) = match gltf::import(path) {
            Ok(result) => result,
            Err(err) => return Err(ModelError::LoadingError(err.to_string()))
        };


        let mut meshes = Vec::new();
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                Self::load_node(device, node, &buffers, &mut meshes);
            }
        }

        
        let mut final_materials = Vec::new();
        for material in gltf.materials() {
            final_materials.push(
                Self::load_material(
                    device,
                    queue,
                    path.parent().unwrap(),
                    material,
                    &buffers,
                    texture_bind_group_layout,
                ).await
            );
        }

        Ok(Model3D {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            meshes,
            materials: final_materials,
        })
    }

    fn load_node(
        device: &wgpu::Device,
        node: gltf::Node<'_>,
        buffers: &[gltf::buffer::Data],
        meshes: &mut Vec<Mesh3D>,
    ) {
        for mesh in node.mesh() {
            for primitive in mesh.primitives() {
                meshes.push(Self::load_primitive(
                    device,
                    mesh.name(),
                    buffers,
                    primitive,
                ));
            }
        }

        for child in node.children() {
            Self::load_node(device, child, buffers, meshes)
        }
    }

   async fn load_material(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model_dir: &Path,
        material: gltf::Material<'_> ,
        buffers: &[gltf::buffer::Data],
        texture_bind_group_layout: &BindGroupLayout,
    ) -> wgpu::BindGroup {
        let pbr = material.pbr_metallic_roughness();


        let diffuse_texture = if pbr.base_color_texture().is_some() {
            let tex = pbr.base_color_texture().unwrap();

            match tex.texture().source().source() {
                gltf::image::Source::View { view, mime_type: _ } => {
                   Texture::from_memory_to_image(device, queue, &buffers[view.buffer().index()], None).unwrap()
                }
                gltf::image::Source::Uri { uri, mime_type: _ } => {
                    dbg!(uri);
                   Texture::from_image(device, queue, &image::open(model_dir.join(uri)).unwrap(), None).unwrap()
                }
            }
        } else {
            Texture::from_color(device, queue, [255, 255, 255, 255], 128, 128, None).unwrap()
        };

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
            ],
            label: material.name(),
        })
    }

    fn load_primitive(
        device: &wgpu::Device,
        name: Option<&str>,
        buffers: &[gltf::buffer::Data],
        primitive: gltf::Primitive,
    ) -> Mesh3D {
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let mut vertices = Vec::new();
        if let Some(vertex_attribute) = reader.read_positions() {
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

        let mut indices = Vec::new();
        if let Some(indices_raw) = reader.read_indices() {
            indices.append(&mut indices_raw.into_u32().collect());
        }
        
        Mesh3D::new(
            device,
            &vertices,
            &indices,
            primitive.material().index().unwrap_or(0),
            name,
        )
    }
}
