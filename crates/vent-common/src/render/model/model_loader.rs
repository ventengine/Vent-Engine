use crate::render::Vertex3D;
use russimp::scene::PostProcess;

pub struct ModelLoader3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,

    pub materials: Vec<russimp::material::Material>,
}

impl ModelLoader3D {
    #[inline]
    #[must_use]
    pub fn load(path: &str) -> Self {
        let scene = russimp::scene::Scene::from_file(
            path,
            vec![
                PostProcess::CalculateTangentSpace,
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
                PostProcess::OptimizeMeshes,
                PostProcess::OptimizeGraph,
                PostProcess::ImproveCacheLocality,
            ],
        )
        .unwrap();

        let mut vertices = Vec::with_capacity(scene.meshes.iter().map(|m| m.vertices.len()).sum());
        let mut indices = Vec::new();
        indices.reserve(scene.meshes.iter().map(|m| m.faces.len() * 3).sum());

        for mesh in &scene.meshes {
            indices.extend(mesh.faces.iter().flat_map(|face| face.0.iter().copied()));

            vertices.extend(mesh.vertices.iter().map(|vertex| Vertex3D {
                _pos: [vertex.x as f32, vertex.y as f32, vertex.z as f32],
                _tex_coord: [0.0, 0.0],
            }));
        }
        let mats = scene.materials;

        Self {
            vertices,
            indices,
            materials: mats,
        }
    }
}
