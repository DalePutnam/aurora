use std::f32;
use std::fmt;

use na::Matrix4;
use na::Vector4;
use util::math;
use Ray;

#[derive(fmt::Debug)]
pub struct BoundingBox
{
	lower_point: Vector4<f32>,
	upper_point: Vector4<f32>,
}

impl BoundingBox
{
	pub fn new(lower_point: &Vector4<f32>, upper_point: &Vector4<f32>) -> Self
	{
		BoundingBox {
			lower_point: *lower_point,
			upper_point: *upper_point,
		}
	}

	pub fn get_extents(&self) -> (&Vector4<f32>, &Vector4<f32>)
	{
		(&self.lower_point, &self.upper_point)
	}

	pub fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> bool
	{
		let point = transform * ray.point();
		let origin = transform * ray.origin();

		let ray_direction = point - origin;
		let inv_direction = Vector4::repeat(1.0).component_div(&ray_direction);

		let min = (self.lower_point.x - origin.x) * inv_direction.x;
		let max = (self.upper_point.x - origin.x) * inv_direction.x;

		let (mut t_min, mut t_max) = if inv_direction.x >= 0.0 {
			(min, max)
		} else {
			(max, min)
		};

		let min = (self.lower_point.y - origin.y) * inv_direction.y;
		let max = (self.upper_point.y - origin.y) * inv_direction.y;

		let (ty_min, ty_max) = if inv_direction.y >= 0.0 {
			(min, max)
		} else {
			(max, min)
		};

		if (t_min > ty_max) || (ty_min > t_max) {
			return false;
		}

		if ty_min > t_min {
			t_min = ty_min;
		}

		if ty_max < t_max {
			t_max = ty_max;
		}

		let min = (self.lower_point.z - origin.z) * inv_direction.z;
		let max = (self.upper_point.z - origin.z) * inv_direction.z;

		let (tz_min, tz_max) = if inv_direction.z >= 0.0 {
			(min, max)
		} else {
			(max, min)
		};

		if (t_min > tz_max) || (tz_min > t_max) {
			return false;
		}

		if tz_min > t_min {
			t_min = tz_min;
		}

		if tz_max < t_max {
			t_max = tz_max;
		}

		if !math::far_from_zero_pos(t_min) && !math::far_from_zero_pos(t_max) {
			return false;
		}

		true
	}
}
