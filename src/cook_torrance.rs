use na::{Vector3, Vector4};
use traits::BSDF;
use util::math;
use std::f32;
use Hit;
use Light;
use Ray;
use Scene;

pub struct CookTorrance {
    colour: Vector3<f32>,
    diffuse: f32,
    roughness: f32,
    refractive_index: f32,
}

impl CookTorrance {
    pub fn new(colour: Vector3<f32>, diffuse: f32, roughness: f32, refractive_index: f32) -> Self {
        CookTorrance {
            colour: colour,
            diffuse: diffuse,
            roughness: roughness * roughness,
            refractive_index: refractive_index,
        }
    }

    fn calculate_diffuse(
        &self,
        contact_point: &Vector4<f32>,
        normal: &Vector4<f32>,
        light: &Light,
    ) -> Vector3<f32> {
        if self.diffuse < math::EPSILON {
            return Vector3::new(0.0, 0.0, 0.0)
        }

        let light_vector = light.get_position() - contact_point;
        let distance = light_vector.dot(&light_vector).sqrt();
        let diffuse = f32::max(light_vector.normalize().dot(&normal.normalize()), 0.0) * self.diffuse;

        light.attenuate(distance).component_mul(&self.colour) * diffuse
    }

    fn calculate_specular(
        &self,
        contact_point: &Vector4<f32>,
        eye: &Vector4<f32>,
        normal: &Vector4<f32>,
        light: &Light,
    ) -> Vector3<f32> {
        if 1.0 - self.diffuse < math::EPSILON {
            return Vector3::new(0.0, 0.0, 0.0)
        }

        let light_vector = light.get_position() - contact_point;
        let distance = light_vector.dot(&light_vector).sqrt();

        let v = (eye - contact_point).normalize();
        let l = light_vector.normalize();
        let n = normal.normalize();
        let h = (v + l).normalize();

        let nv = n.dot(&v);

        if nv < 0.05 {
            return Vector3::new(0.0, 0.0, 0.0);
        }

        let d = ggx_distribution(&h, &n, self.roughness);
        let g = ggx_geometry(&v, &l, &h, &n, self.roughness);
        let f = fresnel(&v, &h, self.refractive_index);

        let specular = ((d * g * f) / (4.0 * nv)) * (1.0 - self.diffuse);

        light.attenuate(distance).component_mul(&self.colour) * specular
    }
}

impl BSDF for CookTorrance {
    fn shade_pixel(&self, ray: &Ray, hit: &Hit, scene: &Scene) -> Vector3<f32> {
        let ac = self.colour.component_mul(&scene.get_ambient());
        let mut dc = Vector3::new(0.0, 0.0, 0.0);
        let mut sc = Vector3::new(0.0, 0.0, 0.0);

        let contact_point = ray.origin + (hit.intersect * (ray.point - ray.origin));

        for light in scene.get_lights().iter() {
            let shadow_ray = Ray {
                origin: contact_point,
                point: *light.get_position(),
                id: 0,
                thread_id: 0,
            };

            if let Some((shadow_hit, _)) = scene.check_hit(&shadow_ray) {
                if shadow_hit.intersect <= 1.0 {
                    continue;
                }
            }

            dc += self.calculate_diffuse(&contact_point, &hit.normal, &light);
            sc += self.calculate_specular(&contact_point, &ray.origin, &hit.normal, &light);
        }

        ac + dc + sc
    }
}

fn chi(a: f32) -> f32 {
    if a > 0.0 {
        1.0
    } else {
        0.0
    }
}

fn ggx_distribution(half: &Vector4<f32>, normal: &Vector4<f32>, alpha: f32) -> f32 {
    let a2 = alpha.powi(2);
    let hn = half.dot(&normal);
    let hn2 = hn.powi(2);

    (chi(hn) * a2) / (f32::consts::PI * ((hn2 * a2) + (1.0 - hn2)).powi(2))
}

fn ggx_geometry(
    view: &Vector4<f32>,
    light: &Vector4<f32>,
    half: &Vector4<f32>,
    normal: &Vector4<f32>,
    alpha: f32
) -> f32 {
    ggx_geometry_partial(view, half, normal, alpha)
        * ggx_geometry_partial(light, half, normal, alpha)
}

fn ggx_geometry_partial(
    direction: &Vector4<f32>,
    half: &Vector4<f32>,
    normal: &Vector4<f32>,
    alpha: f32
) -> f32 {
    let a2 = alpha.powi(2);
    let dh = direction.dot(&half);
    let dn = direction.dot(&normal);
    let dn2 = dn.powi(2);
    let tan2 = (1.0 - dn2) / dn2;

    (chi(dh / dn) * 2.0) / (1.0 + (1.0 + (a2 * tan2)).sqrt())
}

fn fresnel(view: &Vector4<f32>, half: &Vector4<f32>, ior: f32) -> f32 {
    let f0 = (ior - 1.0).powi(2) / (ior + 1.0).powi(2);

    f0 + ((1.0 - f0) * (1.0 - view.dot(&half)).powi(5))
}