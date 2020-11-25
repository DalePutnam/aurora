use na::{Matrix4, Vector4};
use std::sync::Arc;

use traits::Primitive;
use BoundingBox;
use Hit;
use Material;
use Ray;

pub struct Object {
    _id: u64,
    transform: Matrix4<f32>,
    bounding_box: Box<BoundingBox>,
    primitive: Arc<dyn Primitive>,
    material: Arc<Material>,
}

impl Object {
    pub fn new(
        id: u64,
        transform: &Matrix4<f32>,
        primitive: Arc<dyn Primitive>,
        material: Arc<Material>,
    ) -> Self {
        // Get min/max coordinates in model space
        let (min, max) = primitive.get_extents();

        // Use the min/max coordinates to calculate the eight vertices
        // of a bounding box in model space, then transform those vertices
        // to world space.
        let mut world_vertices: Vec<Vector4<f32>> = Vec::new();

        world_vertices.push(transform * Vector4::new(min.x, min.y, min.z, 1.0));
        world_vertices.push(transform * Vector4::new(min.x, min.y, max.z, 1.0));
        world_vertices.push(transform * Vector4::new(min.x, max.y, min.z, 1.0));
        world_vertices.push(transform * Vector4::new(min.x, max.y, max.z, 1.0));
        world_vertices.push(transform * Vector4::new(max.x, min.y, min.z, 1.0));
        world_vertices.push(transform * Vector4::new(max.x, min.y, max.z, 1.0));
        world_vertices.push(transform * Vector4::new(max.x, max.y, min.z, 1.0));
        world_vertices.push(transform * Vector4::new(max.x, max.y, max.z, 1.0));

        // Find the new min/max points in world space so that we can create
        // an axis-aligned bounding box in world space
        let mut world_max =
            Vector4::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);
        let mut world_min = Vector4::new(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);

        for vertex in &world_vertices {
            world_min.x = f32::min(world_min.x, vertex.x);
            world_min.y = f32::min(world_min.y, vertex.y);
            world_min.z = f32::min(world_min.z, vertex.z);

            world_max.x = f32::max(world_max.x, vertex.x);
            world_max.y = f32::max(world_max.y, vertex.y);
            world_max.z = f32::max(world_max.z, vertex.z);
        }

        let bounding_box = BoundingBox::new(&world_min, &world_max);

        Object {
            _id: id,
            bounding_box: Box::new(bounding_box),
            transform: transform.try_inverse().unwrap(), // We need the world to model matrix here
            primitive: primitive,
            material: material,
        }
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)> {
        if self.bounding_box.hit(ray) {
            if let Some(hit) = self.primitive.hit(ray, &self.transform) {
                Some((hit, &self.material))
            } else {
                None
            }
        } else {
            None
        }
    }
}
