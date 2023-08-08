use std::path::Path;

use wgpu::{util::DeviceExt, BindGroupLayout};

use crate::{Model3D, Texture, Vertex3D};

use super::{Material, Mesh3D, ModelError};

pub(crate) struct OBJLoader {}

impl OBJLoader {
    pub async fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Result<Model3D, ModelError> {
        let (models, materials) = match tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS) {
            Ok(r) => r,
            Err(e) => return Err(ModelError::LoadingError(e.to_string())),
        };

        let materials = match materials {
            Ok(r) => r,
            Err(e) => {
                return Err(ModelError::LoadingError(format!(
                    "Failed to load MTL file, {}",
                    e
                )))
            }
        };

        let meshes = models
            .into_iter()
            .map(|model| Self::load_mesh(device, &model.name, &model.mesh))
            .collect::<Vec<_>>();

        let mut final_materials = Vec::with_capacity(materials.len());
        for material in materials {
            final_materials.push(
                Self::load_material(
                    device,
                    queue,
                    path.parent().unwrap(),
                    material,
                    texture_bind_group_layout,
                )
                .await,
            );
        }

        Ok(Model3D {
            meshes,
            materials: final_materials,
        })
    }

    async fn load_material(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model_dir: &Path,
        material: tobj::Material,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> wgpu::BindGroup {
        let diffuse_texture = if let Some(texture) = material.diffuse_texture {
            Texture::from_image(
                device,
                queue,
                &image::open(model_dir.join(&texture)).unwrap(),
                None,
                Some(&texture),
            )
        } else {
            Texture::from_color(device, queue, [255, 255, 255, 255], 128, 128, None)
        };
        let diffuse = material.diffuse.unwrap_or([1.0, 1.0, 1.0]);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(&Material {
                base_color: [diffuse[0], diffuse[1], diffuse[2], 1.0],
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
            label: Some(&material.name),
        })
    }

    fn load_mesh(device: &wgpu::Device, name: &str, mesh: &tobj::Mesh) -> Mesh3D {
        let vertices = (0..mesh.positions.len() / 3)
            .map(|i| Vertex3D {
                position: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ],
                tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                normal: [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ],
            })
            .collect::<Vec<_>>();
        Mesh3D::new(
            device,
            &vertices,
            &mesh.indices,
            mesh.material_id.unwrap_or(0),
            Some(name),
        )
    }
}
