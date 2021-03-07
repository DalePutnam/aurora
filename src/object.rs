use na::Matrix4;
use std::sync::Arc;
use std::fmt;

use traits::Primitive;
use BoundingBox;
use Hit;
use Material;
use Ray;

#[derive(fmt::Debug)]
pub struct Object {
    _name: String,
    _id: u64,
    transform: Matrix4<f32>,
    bounding_box: Box<BoundingBox>,
    primitive: Arc<dyn Primitive>,
    material: Arc<Material>,
}

impl Object {
    pub fn new(
        name: String,
        id: u64,
        transform: &Matrix4<f32>,
        primitive: Arc<dyn Primitive>,
        material: Arc<Material>,
    ) -> Self {
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
        }
    }

    pub fn get_name(&self) -> &String {
        &self._name
    }

    pub fn get_bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    pub fn get_transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)> {
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
}
