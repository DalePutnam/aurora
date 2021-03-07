use std::fmt;

use na::Vector3;
use Hit;
use Ray;
use Scene;

pub trait BSDF: Send + Sync + fmt::Debug
{
	fn shade_pixel(&self, ray: &Ray, hit: &Hit, scene: &Scene) -> Vector3<f32>;
}
