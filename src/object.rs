use std::borrow::Borrow;
use std::cell::Cell;
use std::fmt;
use std::sync::Arc;

use linalg::Transform;
use na::Matrix4;
use primitives::BoundingBox;
use primitives::Intersection;
use primitives::Primitive;
use shading::Material;
use thread_local::ThreadLocal;
use Ray;

#[derive(fmt::Debug)]
pub struct Object
{
    name: String,
    transform: Transform,
    bounding_box: Box<BoundingBox>,
    primitive: Arc<dyn Primitive>,
    material: Arc<dyn Material>,
    last_seen_ray: ThreadLocal<Cell<Option<u64>>>,
}

impl Object
{
    pub fn new(
        name: String,
        transform: &Matrix4<f32>,
        primitive: Arc<dyn Primitive>,
        material: Arc<dyn Material>,
    ) -> Self
    {
        // Get min/max coordinates in model space
        let (min, max) = primitive.get_extents();
        let bounding_box = BoundingBox::new(&min, &max);

        Object {
            name: name,
            bounding_box: Box::new(bounding_box),
            transform: Transform::from(transform.try_inverse().unwrap()), // We need the world to model matrix here
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

    pub fn get_transform(&self) -> &Transform
    {
        &self.transform
    }

    pub fn intersect(&self, ray: &Ray) -> Option<(Intersection, &dyn Material)>
    {
        if self.ray_previously_visited(ray) {
            return None;
        }

        let local_origin = self.transform * ray.origin();
        let local_direction = self.transform * ray.direction();

        if self.bounding_box.hit(ray, &self.transform) {
            if let Some(intersection) = self.primitive.intersect(&local_origin, &local_direction) {
                Some((
                    Intersection {
                        t: intersection.t,
                        normal: self.transform.inverse() * intersection.normal,
                        uv: intersection.uv,
                    },
                    self.material.borrow(),
                ))
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
