use std::f32;
use std::fmt;

use na::Matrix4;
use na::Vector3;
use na::Vector4;
use primitives::Primitive;
use util::math;
use Hit;
use Ray;

#[derive(fmt::Debug)]
pub struct NonhierSphere
{
	position: Vector4<f32>,
	radius: f32,
}

impl NonhierSphere
{
	pub fn new(position: Vector3<f32>, radius: f32) -> Self
	{
		NonhierSphere {
			position: Vector4::new(position.x, position.y, position.z, 1.0),
			radius: radius,
		}
	}
}

impl Primitive for NonhierSphere
{
	fn hit(&self, ray: &Ray, transform: Matrix4<f32>) -> Option<Hit>
	{
		sphere_intersect(self.position, self.radius, ray, transform)
	}

	fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>)
	{
		(
			self.position.add_scalar(-self.radius),
			self.position.add_scalar(self.radius),
		)
	}
}

#[derive(fmt::Debug)]
pub struct Sphere {}

impl Sphere
{
	pub fn new() -> Self
	{
		Sphere {}
	}
}

impl Primitive for Sphere
{
	fn hit(&self, ray: &Ray, transform: Matrix4<f32>) -> Option<Hit>
	{
		sphere_intersect(Vector4::new(0.0, 0.0, 0.0, 1.0), 1.0, ray, transform)
	}

	fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>)
	{
		(
			Vector4::new(-1.0, -1.0, -1.0, 1.0),
			Vector4::new(1.0, 1.0, 1.0, 1.0),
		)
	}
}

fn sphere_intersect(
	position: Vector4<f32>,
	radius: f32,
	ray: &Ray,
	transform: Matrix4<f32>,
) -> Option<Hit>
{
	let point = transform * ray.point();
	let origin = transform * ray.origin();

	let po = point - origin;
	let oc = origin - position;

	let a = po.dot(&po);
	let b = po.dot(&oc) * 2.0;
	let c = oc.dot(&oc) - (radius * radius);

	match math::quadratic_roots(a, b, c) {
		math::QuadRoots::Zero | math::QuadRoots::One(_) => None,
		math::QuadRoots::Two(root_one, root_two) => {
			if !math::far_from_zero_pos(root_one) && !math::far_from_zero_pos(root_two) {
				None
			} else {
				let t = if root_one <= root_two {
					if math::far_from_zero_pos(root_one) {
						root_one
					} else {
						root_two
					}
				} else {
					if math::far_from_zero_pos(root_two) {
						root_two
					} else {
						root_one
					}
				};

				let mut n = (origin + (t * po)) - position;

				// Invert normal if inside sphere
				if n.dot(&(origin - point)) < 0.0 {
					n = -n;
				}

				n = math::transform_normals(n, transform);

				Some(Hit {
					normal: n,
					intersect: t,
					uv: (0.0, 0.0),
				})
			}
		}
	}
}

#[derive(fmt::Debug)]
pub struct NonhierBox
{
	position: Vector4<f32>,
	size: f32,
}

impl NonhierBox
{
	pub fn new(position: Vector3<f32>, size: f32) -> Self
	{
		NonhierBox {
			position: Vector4::new(position.x, position.y, position.z, 1.0),
			size: size,
		}
	}
}

impl Primitive for NonhierBox
{
	fn hit(&self, ray: &Ray, transform: Matrix4<f32>) -> Option<Hit>
	{
		box_intersect(self.position, self.size, ray, transform)
	}

	fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>)
	{
		(self.position, self.position.add_scalar(self.size))
	}
}

#[derive(fmt::Debug)]
pub struct Cube {}

impl Cube
{
	pub fn new() -> Self
	{
		Cube {}
	}
}

impl Primitive for Cube
{
	fn hit(&self, ray: &Ray, transform: Matrix4<f32>) -> Option<Hit>
	{
		box_intersect(Vector4::new(0.0, 0.0, 0.0, 1.0), 1.0, ray, transform)
	}

	fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>)
	{
		(
			Vector4::new(0.0, 0.0, 0.0, 1.0),
			Vector4::new(1.0, 1.0, 1.0, 1.0),
		)
	}
}

fn box_intersect(
	position: Vector4<f32>,
	size: f32,
	ray: &Ray,
	transform: Matrix4<f32>,
) -> Option<Hit>
{
	enum Faces
	{
		Front,
		Back,
		Top,
		Bottom,
		Left,
		Right,
	}

	let point = transform * ray.point();
	let origin = transform * ray.origin();

	let ray_direction = point - origin;
	let inv_direction = Vector4::repeat(1.0).component_div(&ray_direction);

	let min = (position.x - origin.x) * inv_direction.x;
	let max = (position.x + size - origin.x) * inv_direction.x;

	let (mut t_min, mut face_min, mut t_max, mut face_max) = if inv_direction.x >= 0.0 {
		(min, Faces::Left, max, Faces::Right)
	} else {
		(max, Faces::Right, min, Faces::Left)
	};

	let min = (position.y - origin.y) * inv_direction.y;
	let max = (position.y + size - origin.y) * inv_direction.y;

	let (ty_min, y_min_face, ty_max, y_max_face) = if inv_direction.y >= 0.0 {
		(min, Faces::Bottom, max, Faces::Top)
	} else {
		(max, Faces::Top, min, Faces::Bottom)
	};

	if (t_min > ty_max) || (ty_min > t_max) {
		return None;
	}

	if ty_min > t_min {
		t_min = ty_min;
		face_min = y_min_face;
	}

	if ty_max < t_max {
		t_max = ty_max;
		face_max = y_max_face;
	}

	let min = (position.z - origin.z) * inv_direction.z;
	let max = (position.z + size - origin.z) * inv_direction.z;

	let (tz_min, z_face_min, tz_max, z_face_max) = if inv_direction.z >= 0.0 {
		(min, Faces::Back, max, Faces::Front)
	} else {
		(max, Faces::Front, min, Faces::Back)
	};

	if (t_min > tz_max) || (tz_min > t_max) {
		return None;
	}

	if tz_min > t_min {
		t_min = tz_min;
		face_min = z_face_min;
	}

	if tz_max < t_max {
		t_max = tz_max;
		face_max = z_face_max;
	}

	let (intersect, face) = if math::far_from_zero_pos(t_min) {
		(t_min, face_min)
	} else if math::far_from_zero_pos(t_max) {
		(t_max, face_max)
	} else {
		return None;
	};

	let local_normal = match face {
		Faces::Right => Vector4::new(1.0, 0.0, 0.0, 0.0),
		Faces::Left => Vector4::new(-1.0, 0.0, 0.0, 0.0),
		Faces::Top => Vector4::new(0.0, 1.0, 0.0, 0.0),
		Faces::Bottom => Vector4::new(0.0, -1.0, 0.0, 0.0),
		Faces::Front => Vector4::new(0.0, 0.0, 1.0, 0.0),
		Faces::Back => Vector4::new(0.0, 0.0, -1.0, 0.0),
	};

	let world_normal = math::transform_normals(local_normal, transform);

	// TODO: UV value calculation

	Some(Hit {
		intersect: intersect,
		normal: world_normal,
		uv: (0.0, 0.0),
	})
}
