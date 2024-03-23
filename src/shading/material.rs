use std::fmt;

use na::Vector3;
use na::Vector4;

pub trait Material: Send + Sync + fmt::Debug
{
    // Path Tracing Interface
    fn bsdf(&self, w_in: &Vector4<f32>, w_out: &Vector4<f32>) -> Vector3<f32>;
    fn sample_bsdf(&self, w_out: &Vector4<f32>, u: (f32, f32))
        -> (Vector3<f32>, Vector4<f32>, f32);
    fn pdf(&self, w_in: &Vector4<f32>) -> f32;

    // Old naive ray tracing interface
    fn ambient_component(&self) -> Vector3<f32>;
    fn diffuse_component(&self, light: Vector4<f32>, normal: Vector4<f32>) -> Vector3<f32>;
    fn specular_component(
        &self,
        view: Vector4<f32>,
        light: Vector4<f32>,
        normal: Vector4<f32>,
    ) -> Vector3<f32>;
}
