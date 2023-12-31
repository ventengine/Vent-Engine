use std::path::Path;

use ash::vk;
use vent_rendering::{image::VulkanImage, instance::VulkanInstance, Vertex3D};

use crate::Model3D;

use super::{Material, Mesh3D, ModelError};

pub(crate) struct OBJLoader {}

impl OBJLoader {
    pub async fn load(instance: &VulkanInstance, path: &Path) -> Result<Model3D, ModelError> {
        let (models, materials) = match tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS) {
            Ok(r) => r,
            Err(e) => return Err(ModelError::LoadingError(format!("{}", e))),
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

        let mut meshes = vec![];
        for model in models {
            let mesh = Self::load_mesh(&model.mesh);

            let matieral = Self::load_material(
                instance,
                path.parent().unwrap(),
                &materials[model.mesh.material_id.unwrap()],
            );

            meshes.push(Mesh3D::new(
                &instance.device,
                &instance.memory_allocator,
                &mesh.0,
                mesh.1,
                Some(matieral), // TODO
                Some(&model.name),
            ));
        }

        Ok(Model3D { meshes })
    }

    fn load_material(
        instance: &VulkanInstance,
        model_dir: &Path,
        material: &tobj::Material,
    ) -> Material {
        let diffuse_texture = if let Some(texture) = &material.diffuse_texture {
            VulkanImage::from_image(
                &instance.device,
                image::open(model_dir.join(texture)).unwrap(),
                instance.command_pool,
                &instance.memory_allocator,
                instance.graphics_queue,
                None,
            )
        } else {
            VulkanImage::from_color(
                &instance.device,
                instance.command_pool,
                &instance.memory_allocator,
                instance.graphics_queue,
                [255, 255, 255, 255],
                vk::Extent2D {
                    width: 128,
                    height: 128,
                },
            )
        };
        let base_color = material.diffuse.unwrap_or([1.0, 1.0, 1.0]);
        // OBJ does not have an Alpha :c
        let base_color = [base_color[0], base_color[1], base_color[2], 1.0];

        Material {
            diffuse_texture,
            base_color,
        }
    }

    fn load_mesh(mesh: &tobj::Mesh) -> (Vec<Vertex3D>, &[u32]) {
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
        (vertices, &mesh.indices)
    }
}
