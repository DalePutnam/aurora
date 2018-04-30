use na::{Matrix4, Vector4};
use core::Ray;

pub trait Primitive: Send + Sync {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>, intersect: &mut f32, normal: &mut Vector4<f32>, u: &mut f32, v: &mut f32) -> bool;
}