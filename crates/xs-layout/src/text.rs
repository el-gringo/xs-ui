#[derive(PartialEq, Clone, Debug)]
pub struct TextVertices {
    pub position: (Vec<f32>, usize),
    pub texture_coordinate: (Vec<f32>, usize),
    pub num_vertices: usize,
    pub width: f32,
    pub height: f32,
}
