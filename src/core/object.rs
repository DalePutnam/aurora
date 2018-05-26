use std::sync::Arc;
use na::Matrix4;
use core::traits::Primitive;
use core::{Material, Ray, Hit};

pub struct Object {
    id: u64,
    transform: Matrix4<f32>,
    inverse_transform: Matrix4<f32>,
    primitive: Arc<Box<Primitive>>,
    material: Arc<Box<Material>>,
}

impl Object {
    pub fn new(id: u64, transform: &Matrix4<f32>, primitive: &Arc<Box<Primitive>>, material: &Arc<Box<Material>>) -> Self {
        // The transform should never not be invertable
        Object {
            id: id,
            transform: *transform,
            inverse_transform: transform.try_inverse().unwrap(),
            primitive: Arc::clone(primitive),
            material: Arc::clone(material),
        }
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, Arc<Box<Material>>)> {
        if let Some(hit) = self.primitive.hit(ray, &self.inverse_transform) {
            Some((hit, Arc::clone(&self.material)))
        } else {
            None
        }
   }
}
