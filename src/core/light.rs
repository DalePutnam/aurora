use na::{Vector4, Vector3};

#[derive(Clone)]
pub struct Light {
    position: Vector4<f32>,
    colour: Vector3<f32>,
    falloff: Vector3<f32>,
}

impl Light {
    pub fn new(position: &Vector3<f32>, colour: &Vector3<f32>, falloff: &Vector3<f32>) -> Self {
        Light {
            position: Vector4::<f32>::new(position.x, position.y, position.z, 1.0),
            colour: *colour,
            falloff: *falloff,
        }
    }

    pub fn get_position(&self) -> &Vector4<f32> {
        &self.position
    }

    pub fn get_colour(&self) -> &Vector3<f32> {
        &self.colour
    }

    pub fn get_falloff(&self) -> &Vector3<f32> {
        &self.falloff
    }
}