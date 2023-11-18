use std::{mem::size_of, path::Path};

use ash::vk;
use vent_rendering::{
    buffer::VulkanBuffer, image::VulkanImage, instance::VulkanInstance, Vertex3D,
};

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

        let meshes = models
            .into_iter()
            .map(|model| Self::load_mesh(instance, &model.name, &model.mesh))
            .collect::<Vec<_>>();

        let _descriptor_sets = materials
            .into_iter()
            .map(|material| Self::load_material(instance, path.parent().unwrap(), &material))
            .collect::<Vec<_>>();

        Ok(Model3D { meshes })
    }

    fn load_material(
        instance: &VulkanInstance,
        model_dir: &Path,
        material: &tobj::Material,
    ) -> Vec<vk::W> {
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
                [255, 255, 255, 255],
                vk::Extent2D {
                    width: 128,
                    height: 128,
                },
            )
        };
        let diffuse = material.diffuse.unwrap_or([1.0, 1.0, 1.0]);

        let binding = Material {
            base_color: [diffuse[0], diffuse[1], diffuse[2], 1.0],
        };

        let mut uniform_buffers = vec![];
        Self::create_uniform_buffers(instance, &binding, &mut uniform_buffers);

        Self::write_sets(instance, diffuse_texture, &uniform_buffers)
    }

    fn create_uniform_buffers(
        instance: &VulkanInstance,
        material: &Material,
        uniform_buffers: &mut Vec<VulkanBuffer>,
    ) {
        for _ in 0..instance.swapchain_images.len() {
            let buffer = unsafe {
                VulkanBuffer::new_init_type(
                    &instance.device,
                    &instance.memory_allocator,
                    size_of::<Material>() as vk::DeviceSize,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    material,
                )
            };
            uniform_buffers.push(buffer)
        }
    }

    fn write_sets(
        instance: &VulkanInstance,
        diffuse_texture: VulkanImage,
        uniforms_buffers: &Vec<VulkanBuffer>,
    ) -> Vec<vk::DescriptorSet> {
        let descriptor_sets = VulkanInstance::allocate_descriptor_sets(
            &instance.device,
            instance.descriptor_pool,
            instance.descriptor_set_layout,
            uniforms_buffers.len(),
        );

        for (i, &_descritptor_set) in descriptor_sets.iter().enumerate() {
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(diffuse_texture.image_view)
                .sampler(diffuse_texture.sampler)
                .build();

            let _sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&[image_info]);

            let buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(uniforms_buffers[i].buffer)
                .offset(0)
                .range(size_of::<Material>() as vk::DeviceSize)
                .build();

            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&[buffer_info])
                .build();

            unsafe {
                instance.device.update_descriptor_sets(&[ubo_write], &[]);
            }
        }
        descriptor_sets
    }

    fn load_mesh(
        instance: &VulkanInstance,
        // bind_group: Vec<vk::DescriptorSet>, TODO
        name: &str,
        mesh: &tobj::Mesh,
    ) -> Mesh3D {
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
            &instance.device,
            &instance.memory_allocator,
            &vertices,
            &mesh.indices,
            None, // TODO
            Some(name),
        )
    }
}
