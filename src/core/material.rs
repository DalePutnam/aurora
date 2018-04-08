use na::Vector3;

pub struct Material {
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
    shininess: f64,
}

impl Material {
    fn new(diffuse: Vector3<f32>, specular: Vector3<f32>, shininess: f64) -> Self {
        Material {
            diffuse: diffuse,
            specular: specular,
            shininess: shininess,
        }
    }

    fn get_diffuse(&self) -> &Vector3<f32> {
        &self.diffuse
    }

    fn get_specular(&self) -> &Vector3<f32> {
        &self.specular
    }

    fn get_shininess(&self) -> f64 {
        self.shininess
    }
}