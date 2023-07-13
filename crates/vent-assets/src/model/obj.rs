use std::path::Path;

use vent_common::render::Vertex3D;

use super::{Mesh3D, ModelError};

pub struct OBJLoader {}

impl OBJLoader {
    pub fn load(device: &wgpu::Device, path: &Path) -> Result<Vec<Mesh3D>, ModelError> {
        let (models, _materials) = match tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS) {
            Ok(r) => r,
            Err(e) => return Err(ModelError::LoadingError(e.to_string())),
        };

        let mut meshes = Vec::new();
        for model in models {
            meshes.push(Self::load_mesh(device, &model.name, &model.mesh));
        }
        Ok(meshes)
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
            })
            .collect::<Vec<_>>();

        Mesh3D::new(device, &vertices, &mesh.indices, name)
    }
}
