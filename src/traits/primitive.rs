use na::Matrix4;
use Ray;
use Hit;

pub trait Primitive: Send + Sync {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit>;
}