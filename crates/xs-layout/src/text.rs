#[derive(PartialEq, Clone, Debug, Default)]
pub struct TextVertices {
    pub vertices: Vec<f32>,
    pub width: f32,
    pub height: f32,
    pub font_scale: f32,
    pub distance_range: f32,
}
