use crate::render::Vertex3D;
use russimp::scene::PostProcess;

pub struct ModelLoader3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl ModelLoader3D {
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
        let indices = scene
            .meshes
            .iter()
            .flat_map(|m| m.faces.iter().flat_map(|face| face.0.iter().copied()))
            .collect::<Vec<_>>();

        for mesh in scene.meshes {
            vertices.extend(mesh.vertices.iter().map(|vertex| Vertex3D {
                _pos: [vertex.x, vertex.y, vertex.z],
                _tex_coord: [0.0, 0.0],
            }));
        }

        Self { vertices, indices }
    }


}
