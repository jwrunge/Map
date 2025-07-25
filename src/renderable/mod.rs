pub mod renderable;
mod transforms;
pub mod vertex;

use renderable::Renderable;
use transforms::Transform;
use vertex::Vertex;

use crate::renderable::vertex::VertexProvider;

pub struct Triangle {
    vertices: [Vertex; 3],
    pub transform: Transform,
}

impl Triangle {
    pub fn new() -> Self {
        Triangle {
            vertices: [
                Vertex {
                    position: [0.0, 0.5, 0.0],
                    color: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [-0.5, -0.5, 0.0],
                    color: [0.0, 1.0, 0.0],
                },
                Vertex {
                    position: [0.5, -0.5, 0.0],
                    color: [0.0, 0.0, 1.0],
                },
            ],
            transform: Transform::new(),
        }
    }
}

impl Renderable for Triangle {
    fn update(&mut self, delta: f32) {
        self.transform.rotate_degrees(0.0, 0.0, 60.0 * delta);
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}

impl VertexProvider for Triangle {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}
