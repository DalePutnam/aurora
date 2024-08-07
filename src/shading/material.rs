use std::fmt;

use linalg::Vector;
use na::Vector3;

#[derive(Clone, Copy)]
pub struct UV(pub f32, pub f32);

pub trait Material: Send + Sync + fmt::Debug
{
    // Path Tracing Interface
    fn bsdf(&self, w_in: &Vector, w_out: &Vector) -> Vector3<f32>;
    fn sample_bsdf(&self, w_out: &Vector, u: (f32, f32)) -> (Vector3<f32>, Vector, f32);
    fn pdf(&self, w_in: &Vector) -> f32;
}
