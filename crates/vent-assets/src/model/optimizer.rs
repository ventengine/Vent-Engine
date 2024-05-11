use vent_rendering::Vertex3D;

pub fn optimize_vertices(mut vertices: Vec<Vertex3D>) -> Vec<Vertex3D> {
    // deduplicate
    let original_size = vertices.len();
    vertices.dedup();
    let new_size = vertices.len();
    if new_size != original_size {
        log::debug!(
            "Deduped Vertices, Original {}, New {}",
            original_size,
            new_size
        );
    }
    vertices
}
