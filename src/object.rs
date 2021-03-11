use std::cell::Cell;
use std::fmt;
use std::sync::Arc;

use na::Matrix4;
use traits::Primitive;
use BoundingBox;
use Hit;
use Material;
use Ray;

#[derive(fmt::Debug)]
pub struct Object
{
	_name: String,
	_id: u64,
	transform: Matrix4<f32>,
	bounding_box: Box<BoundingBox>,
	primitive: Arc<dyn Primitive>,
	material: Arc<Material>,
	last_ray: Vec<Cell<Option<u64>>>,
}

impl Object
{
	pub fn new(
		name: String,
		id: u64,
		transform: &Matrix4<f32>,
		primitive: Arc<dyn Primitive>,
		material: Arc<Material>,
		num_threads: usize,
	) -> Self
	{
		// Get min/max coordinates in model space
		let (min, max) = primitive.get_extents();
		let bounding_box = BoundingBox::new(&min, &max);

		Object {
			_name: name,
			_id: id,
			bounding_box: Box::new(bounding_box),
			transform: transform.try_inverse().unwrap(), // We need the world to model matrix here
			primitive: primitive,
			material: material,
			last_ray: vec![Cell::new(None); num_threads],
		}
	}

	pub fn get_name(&self) -> &String
	{
		&self._name
	}

	pub fn get_bounding_box(&self) -> &BoundingBox
	{
		&self.bounding_box
	}

	pub fn get_transform(&self) -> &Matrix4<f32>
	{
		&self.transform
	}

	pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)>
	{
		if self.ray_previously_visited(ray) {
			return None;
		}

		if self.bounding_box.hit(ray, &self.transform) {
			if let Some(hit) = self.primitive.hit(ray, &self.transform) {
				Some((hit, &self.material))
			} else {
				None
			}
		} else {
			None
		}
	}

	fn ray_previously_visited(&self, ray: &Ray) -> bool
	{
		if ray.get_thread() >= self.last_ray.len() {
			false
		} else {
			let id_cell = &self.last_ray[ray.get_thread()];
			if let Some(id) = id_cell.replace(Some(ray.get_id())) {
				id == ray.get_id()
			} else {
				false
			}
		}
	}
}

unsafe impl Sync for Object {}