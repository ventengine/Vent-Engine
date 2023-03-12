use crate::render::Vertex3D;
use russimp::scene::PostProcess;

pub struct ModelLoader3D {
    vertices: Vec<Vertex3D>,
    indices: Vec<u32>,
}

impl ModelLoader3D {
    pub fn new(path: &str) -> Self {
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

        let mut vertices: Vec<Vertex3D> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for mesh in scene.meshes {
            vertices.reserve(mesh.vertices.len());

            for vertex in mesh.vertices {
                vertices.push(Vertex3D {
                    _pos: [vertex.x, vertex.y, vertex.z],
                    _tex_coord: [0.0, 0.0],
                })
            }

            indices.reserve(mesh.faces.len());
            for face in mesh.faces {
                for indicie in face.0 {
                    indices.push(indicie);
                }
            }
        }

        Self { vertices, indices }
    }

    pub fn vertices(self) -> Vec<Vertex3D> {
        self.vertices
    }

    pub fn indicies(self) -> Vec<u32> {
        self.indices
    }
}
