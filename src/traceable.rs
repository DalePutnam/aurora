use std::fmt;

use na::Matrix4;

use shading::Material;
use primitives::BoundingBox;
use Hit;
use Ray;

pub trait Traceable: Send + Sync + fmt::Debug
{
	fn check_hit(&self, ray: &Ray) -> Option<(Hit, &dyn Material)>;
	fn get_bounding_box(&self) -> &BoundingBox;
	fn get_transform(&self) -> Matrix4<f32>;
}
