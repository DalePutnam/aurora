use na::Matrix4;
use std::sync::Arc;
use traits::Primitive;
use {Hit, Material, Ray};

pub struct Object {
    _id: u64,
    transform: Matrix4<f32>,
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
        Object {
            _id: id,
            transform: transform.try_inverse().unwrap(), // We need the world to model matrix here
            primitive: primitive,
            material: material,
        }
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)> {
        if let Some(hit) = self.primitive.hit(ray, &self.transform) {
            Some((hit, &self.material))
        } else {
            None
        }
    }
}
