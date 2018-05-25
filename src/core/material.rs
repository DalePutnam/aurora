use na::Vector3;

pub struct Material {
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
    shininess: f32,
}

impl Material {
    pub fn new(diffuse: Vector3<f32>, specular: Vector3<f32>, shininess: f32) -> Self {
        Material {
            diffuse: diffuse,
            specular: specular,
            shininess: shininess,
        }
    }

    pub fn get_diffuse(&self) -> &Vector3<f32> {
        &self.diffuse
    }

    pub fn get_specular(&self) -> &Vector3<f32> {
        &self.specular
    }

    pub fn get_shininess(&self) -> f32 {
        self.shininess
    }
}