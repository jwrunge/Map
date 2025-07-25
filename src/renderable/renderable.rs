use super::transforms::Transform;

pub trait Renderable {
    fn update(&self, delta: f32);
    fn get_transform(&self) -> Transform;
    fn translate(&mut self, translation: [f32; 3]);
    fn rotate(&mut self, rotation: [f32; 4]);
    fn scale(&mut self, scale: [f32; 3]);
}
