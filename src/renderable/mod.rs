mod renderable;
mod transforms;
pub mod vertex;

use renderable::Renderable;
use transforms::Transform;
use vertex::Vertex;

pub struct Triangle {
    vertices: [Vertex; 3],
    pub transform: Transform,
}

impl Renderable for Triangle {
    fn update(&self, delta: f32) {
        // Update logic for the triangle
    }
}
