use std::path::Path;

use wgpu::BindGroupLayout;

use crate::{Vertex3D};

use super::{Mesh3D, ModelError};

pub(crate) struct OBJLoader {}

impl OBJLoader {
    pub fn load(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,                                                                        
        texture_bind_group_layout: BindGroupLayout,
    ) -> Result<Vec<Mesh3D>, ModelError> {
        let (models, materials) = match tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS) {
            Ok(r) => r,
            Err(e) => return Err(ModelError::LoadingError(e.to_string())),
        };

        let materials = match materials {
            Ok(r) => r,
            Err(e) => {
                return Err(ModelError::LoadingError(format!(
                    "Failed loading Materials, {}",
                    e
                )))
            }
        };

        let mut meshes = Vec::with_capacity(models.len());
        for model in models {
            let material: Option<&tobj::Material> = match model.mesh.material_id {
                Some(r) => Some(&materials[r]),
                None => None,
            };
            meshes.push(Self::load_mesh(
                device,
                queue,
                &model.name,
                &model.mesh,
                material,
                &texture_bind_group_layout,
            ));
        }
        Ok(meshes)
    }

    fn load_mesh(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        name: &str,
        mesh: &tobj::Mesh,
        _material: Option<&tobj::Material>,
        _texture_bind_group_layout: &BindGroupLayout,
    ) -> Mesh3D {
        let vertices = (0..mesh.positions.len() / 3)
            .map(|i| Vertex3D {
                position: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ],
                tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
            })
            .collect::<Vec<_>>();
        // let empty_texture = Texture::from_image(device, queue, img, label).unwrap(); 
        // if let Some(material) = material {
        //     if let Some(diffuse) = material.diffuse_texture {

        //     }
        // }
        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: texture_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&empty_texture.view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&empty_texture.sampler),
        //         },
        //     ],
        //     label: None,
        // });
        Mesh3D::new(device, &vertices, &mesh.indices, name)
    }
}
