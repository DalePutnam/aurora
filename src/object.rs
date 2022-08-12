use std::borrow::Borrow;
use std::cell::Cell;
use std::fmt;
use std::sync::Arc;

use na::Matrix4;
use primitives::BoundingBox;
use primitives::Primitive;
use shading::Material;
use thread_local::ThreadLocal;
use Hit;
use Ray;

#[derive(fmt::Debug)]
pub struct Object
{
	name: String,
	transform: Matrix4<f32>,
	bounding_box: Box<BoundingBox>,
	primitive: Arc<dyn Primitive>,
	material: Arc<dyn Material>,
	last_seen_ray: ThreadLocal<Cell<Option<u64>>>,
}

impl Object
{
	pub fn new(
		name: String,
		transform: Matrix4<f32>,
		primitive: Arc<dyn Primitive>,
		material: Arc<dyn Material>,
	) -> Self
	{
		// Get min/max coordinates in model space
		let (min, max) = primitive.get_extents();
		let bounding_box = BoundingBox::new(min, max);

		Object {
			name: name,
			bounding_box: Box::new(bounding_box),
			transform: transform.try_inverse().unwrap(), // We need the world to model matrix here
			primitive: primitive,
			material: material,
			last_seen_ray: ThreadLocal::new(),
		}
	}

	pub fn get_name(&self) -> &String
	{
		&self.name
	}

	pub fn get_bounding_box(&self) -> &BoundingBox
	{
		&self.bounding_box
	}

	pub fn get_transform(&self) -> Matrix4<f32>
	{
		self.transform
	}

	pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &dyn Material)>
	{
		if self.ray_previously_visited(ray) {
			return None;
		}

		if self.bounding_box.hit(ray, self.transform) {
			if let Some(hit) = self.primitive.hit(ray, self.transform) {
				Some((hit, self.material.borrow()))
			} else {
				None
			}
		} else {
			None
		}
	}

	fn ray_previously_visited(&self, ray: &Ray) -> bool
	{
		let last_seen_ray_cell = self.last_seen_ray.get_or(|| Cell::new(None));

		if let Some(last_seen_ray_id) = last_seen_ray_cell.replace(Some(ray.id())) {
			last_seen_ray_id == ray.id()
		} else {
			false
		}
	}
}
