use std::sync::{RwLock, Arc};
use na::{Matrix4, Vector3, Unit};
use core::math::degrees_to_radians;

pub struct SceneNode {
    name: String,
    transform_matrix: Matrix4<f32>,
    inverse_matrix: Matrix4<f32>,
    children: Vec<Arc<RwLock<SceneNode>>>,
}

impl SceneNode {
    pub fn new(name: &str) -> SceneNode {
        SceneNode { 
            name: name.to_string(),
            transform_matrix: Matrix4::<f32>::identity(),
            inverse_matrix: Matrix4::<f32>::identity(),
            children: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn rotate(&mut self, axis: char, angle: f32) {
        println!("Rotating {} degrees on the {} axis", angle, axis);

        let rotation_axis = match axis {
            'x' | 'X' => Vector3::<f32>::new(1.0, 0.0, 0.0),
            'y' | 'Y' => Vector3::<f32>::new(0.0, 1.0, 0.0),
            'z' | 'Z' => Vector3::<f32>::new(0.0, 0.0, 1.0),
             _  => return,
        };

        let rotation_matrix = Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis), degrees_to_radians(angle));
        let new_matrix = rotation_matrix * self.transform_matrix;

        println!("New Matrix is {}", new_matrix);

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn scale(&mut self, amount: &Vector3<f32>) {
        println!("Scaling by {}", amount);

        let scale_matrix = Matrix4::new_nonuniform_scaling(amount);
        let new_matrix = scale_matrix * self.transform_matrix;

        println!("New Matrix is {}", new_matrix);

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn translate(&mut self, amount: &Vector3<f32>) {
        println!("Translating by {}", amount);

        let translate_matrix = Matrix4::new_translation(amount);
        let new_matrix = translate_matrix * self.transform_matrix;

        println!("New Matrix is {}", new_matrix);

        set_transform(&new_matrix, &mut self.transform_matrix, &mut self.inverse_matrix);
    }

    pub fn add_child(&mut self, node: &Arc<RwLock<SceneNode>>) {
        match node.read() {
            Ok(scene_node) => println!("Adding child {} to {}!", scene_node.get_name(), self.name),
            Err(_) => panic!("Children vector is poisoned!"),
        };

        self.children.push(Arc::clone(node));
    }
}

fn set_transform(transform: &Matrix4<f32>, matrix: &mut Matrix4<f32>, inverse_matrix: &mut Matrix4<f32>) {
    *matrix = *transform;
    *inverse_matrix = matrix.try_inverse().unwrap();
}

