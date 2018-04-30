use std::sync::{Arc, Mutex};
use na::Matrix4;
use core::traits::Primitive;
use core::Material;

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
}
