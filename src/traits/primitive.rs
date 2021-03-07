use na::{Matrix4, Vector4};
use Hit;
use Ray;
use std::fmt;

pub trait Primitive: Send + Sync + fmt::Debug {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit>;
    fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>);
}
