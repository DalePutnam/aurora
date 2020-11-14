use std::sync::Arc;
use na::Matrix4;
use core::traits::Primitive;
use core::{Material, Ray, Hit};

pub struct Object {
    _id: u64,
    transform: Matrix4<f32>,
    primitive: Arc<dyn Primitive>,
    material: Material,
}

impl Object {
    pub fn new(id: u64, transform: &Matrix4<f32>, primitive: &Arc<dyn Primitive>, material: &Material) -> Self {
        Object {
            _id: id,
            transform: transform.try_inverse().unwrap(), // We need the world to model matrix here
            primitive: Arc::clone(primitive),
            material: material.clone(),
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
