use na::Vector3;
use Hit;
use Light;
use Material;
use Object;
use Ray;

pub struct Scene {
    objects: Vec<Object>,
    lights: Vec<Light>,
    ambient: Vector3<f32>,
}

impl Scene {
    pub fn new(objects: Vec<Object>, lights: Vec<Light>, ambient: Vector3<f32>) -> Self {
        Scene {
            objects: objects,
            lights: lights,
            ambient: ambient,
        }
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)> {
        let mut min_intersect = f32::INFINITY;
        let mut final_hit: Option<(Hit, &Material)> = None;

        for object in self.objects.iter() {
            if let Some((hit, material)) = object.check_hit(ray) {
                if hit.intersect < min_intersect {
                    min_intersect = hit.intersect;
                    final_hit = Some((hit, &material));
                }
            }
        }

        final_hit
    }

    pub fn get_lights(&self) -> &Vec<Light> {
        &self.lights
    }

    pub fn get_ambient(&self) -> Vector3<f32> {
        self.ambient
    }
}
