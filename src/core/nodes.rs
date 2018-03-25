//use std::cell::RefCell;
use std::sync::{RwLock, Arc};
use na::{Matrix4, Vector3, Unit};
//use rlua::{UserData, UserDataMethods, Value};

static PI: f32 = 3.14159265;

fn degrees_to_radians(angle: f32) -> f32 {
    angle * (PI / 180.0)
}

pub trait Node: Send + Sync {
    fn get_name(&self) -> &String;
    fn rotate(&self, axis: char, angle: f32);
    fn scale(&self, amount: &Vector3<f32>);
    fn translate(&self, amount: &Vector3<f32>);
    fn add_child(&self, node: &Arc<Node>);
}

pub struct SceneNode {
    name: String,
    // transform_matrix: RefCell<Matrix4<f32>>,
    // inverse_matrix: RefCell<Matrix4<f32>>,
    transform_matrix: RwLock<Matrix4<f32>>,
    inverse_matrix: RwLock<Matrix4<f32>>,
    children: RwLock<Vec<Arc<Node>>>,
}

impl SceneNode {
    pub fn new(name: &str) -> SceneNode {
        SceneNode { 
            name: name.to_string(),
            transform_matrix: RwLock::new(Matrix4::<f32>::identity()),
            inverse_matrix: RwLock::new(Matrix4::<f32>::identity()),
            children: RwLock::new(Vec::new())
        }
    }

    fn set_transform(&self, transform: Matrix4<f32>) {
        // let mut transform_matrix = self.transform_matrix.borrow_mut();
        // let mut inverse_matrix = self.inverse_matrix.borrow_mut();

        // *transform_matrix = transform;
        // *inverse_matrix = transform.try_inverse().unwrap();

        let mut transform_matrix = match self.transform_matrix.write() {
            Ok(matrix) => matrix,
            Err(_) => panic!("Transform Matrix is poisoned!"),
        };
        let mut inverse_matrix = match self.inverse_matrix.write() {
            Ok(matrix) => matrix,
            Err(_) => panic!("Inverse Tranform Matrix is poisoned!"),
        };

        *transform_matrix = transform;
        *inverse_matrix = transform.try_inverse().unwrap();
    }
}

impl Node for SceneNode {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn rotate(&self, axis: char, angle: f32) {
        println!("Rotating {} degrees on the {} axis", angle, axis);

        let rotation_axis = match axis {
            'x' | 'X' => Vector3::<f32>::new(1.0, 0.0, 0.0),
            'y' | 'Y' => Vector3::<f32>::new(0.0, 1.0, 0.0),
            'z' | 'Z' => Vector3::<f32>::new(0.0, 0.0, 1.0),
             _  => return,
        };

        let rotation_matrix = Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis), degrees_to_radians(angle));

        let new_matrix = {
            let old_matrix = match self.transform_matrix.read() {
                Ok(matrix) => matrix,
                Err(_) => panic!("Transform Matrix is poisoned!"),
            };
            rotation_matrix * *old_matrix
        };

        println!("New Matrix is {}", new_matrix);

        self.set_transform(new_matrix);
    }

    fn scale(&self, amount: &Vector3<f32>) {
        println!("Scaling by {}", amount);

        let scale_matrix = Matrix4::new_nonuniform_scaling(amount);
        // let new_matrix = {
        //     let old_matrix = self.transform_matrix.borrow();
        //     scale_matrix * *old_matrix
        // };
        let new_matrix = {
            let old_matrix = match self.transform_matrix.read() {
                Ok(matrix) => matrix,
                Err(_) => panic!("Transform Matrix is poisoned!"),
            };
            scale_matrix * *old_matrix
        };

        println!("New Matrix is {}", new_matrix);

        self.set_transform(new_matrix);
    }

    fn translate(&self, amount: &Vector3<f32>) {
        println!("Translating by {}", amount);

        let translate_matrix = Matrix4::new_translation(amount);

        // let new_matrix = {
        //     let old_matrix = self.transform_matrix.borrow();
        //     translate_matrix * *old_matrix
        // };

        let new_matrix = {
            let old_matrix = match self.transform_matrix.read() {
                Ok(matrix) => matrix,
                Err(_) => panic!("Transform Matrix is poisoned!"),
            };
            translate_matrix * *old_matrix
        };

        println!("New Matrix is {}", new_matrix);

        self.set_transform(new_matrix);
    }

    fn add_child(&self, node: &Arc<Node>) {
        println!("Adding child {} to {}!", node.get_name(), self.name);

        let mut children = match self.children.write() {
            Ok(vec) => vec,
            Err(_) => panic!("Children vector is poisoned!"),
        };

        children.push(Arc::clone(node));
    }
}
