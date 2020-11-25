use na::{Matrix4, Vector4};
use Hit;
use Ray;

pub trait Primitive: Send + Sync {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit>;
    fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>);
}
