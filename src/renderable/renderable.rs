use super::transforms::Transform;

pub trait Renderable {
    fn update(&mut self, delta: f32); // Changed to &mut self
    fn get_transform(&self) -> &Transform;
}
