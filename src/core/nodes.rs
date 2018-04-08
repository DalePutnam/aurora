use std::sync::{RwLock, Arc};
use na::{Matrix4, Vector3, Unit};
use core::math::degrees_to_radians;
use core::material::Material;
use core::primitives::Primitive;
use core::ray::{Ray, Hit};

pub struct SceneNode {
    name: String,
    transform_matrix: Matrix4<f32>,
    inverse_matrix: Matrix4<f32>,
    children: Vec<Arc<RwLock<SceneNode>>>,
    primitive: Option<Arc<Primitive>>,
    material: Option<Arc<Material>>
}

impl SceneNode {
    pub fn new(name: &str) -> Self {
        SceneNode { 
            name: name.to_string(),
            transform_matrix: Matrix4::<f32>::identity(),
            inverse_matrix: Matrix4::<f32>::identity(),
            children: Vec::new(),
            primitive: None,
            material: None,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn rotate(&mut self, axis: char, angle: f32) {
        let rotation_axis = match axis {
            'x' | 'X' => Vector3::<f32>::new(1.0, 0.0, 0.0),
            'y' | 'Y' => Vector3::<f32>::new(0.0, 1.0, 0.0),
            'z' | 'Z' => Vector3::<f32>::new(0.0, 0.0, 1.0),
             _  => return,
        };

        let rotation_matrix = Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis), degrees_to_radians(angle));
        let new_matrix = rotation_matrix * self.transform_matrix;

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn scale(&mut self, amount: &Vector3<f32>) {
        let scale_matrix = Matrix4::new_nonuniform_scaling(amount);
        let new_matrix = scale_matrix * self.transform_matrix;

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn translate(&mut self, amount: &Vector3<f32>) {
        let translate_matrix = Matrix4::new_translation(amount);
        let new_matrix = translate_matrix * self.transform_matrix;

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn add_child(&mut self, node: &Arc<RwLock<SceneNode>>) {
        self.children.push(Arc::clone(node));
    }

    pub fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit> {
        None
    }
}

fn set_transform(transform: &Matrix4<f32>, matrix: &mut Matrix4<f32>, inverse_matrix: &mut Matrix4<f32>) {
    *matrix = *transform;
    *inverse_matrix = matrix.try_inverse().unwrap();
}

