use na::Vector3;
use traits::BSDF;
use Hit;
use Ray;
use Scene;

pub struct Material {
    bsdf: Box<dyn BSDF>,
}

impl Material {
    pub fn new<T: BSDF + 'static>(bsdf: T) -> Self {
        Material {
            bsdf: Box::new(bsdf),
        }
    }

    pub fn shade_pixel(&self, ray: &Ray, hit: &Hit, scene: &Scene) -> Vector3<f32> {
        self.bsdf.shade_pixel(ray, hit, scene)
    }
}
