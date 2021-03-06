use std::fmt;

use na::Vector3;
use na::Vector4;
use traits::BSDF;
use Hit;
use Light;
use Ray;
use Scene;

#[derive(fmt::Debug)]
pub struct Phong
{
	diffuse: Vector3<f32>,
	specular: Vector3<f32>,
	shininess: f32,
}

impl Phong
{
	pub fn new(diffuse: Vector3<f32>, specular: Vector3<f32>, shininess: f32) -> Self
	{
		Phong {
			diffuse: diffuse,
			specular: specular,
			shininess: shininess,
		}
	}

	fn calculate_diffuse(
		&self,
		contact_point: &Vector4<f32>,
		normal: &Vector4<f32>,
		light: &Light,
	) -> Vector3<f32>
	{
		if self.diffuse.x != 0.0 || self.diffuse.y != 0.0 || self.diffuse.z != 0.0 {
			let light_vector = light.get_position() - contact_point;
			let distance = light_vector.dot(&light_vector).sqrt();

			let factor = light_vector.normalize().dot(&normal.normalize()).max(0.0);

			light
				.attenuate(distance)
				.component_mul(&self.diffuse)
				.component_mul(&Vector3::repeat(factor))
		} else {
			Vector3::new(0.0, 0.0, 0.0)
		}
	}

	fn calculate_specular(
		&self,
		contact_point: &Vector4<f32>,
		eye: &Vector4<f32>,
		normal: &Vector4<f32>,
		light: &Light,
	) -> Vector3<f32>
	{
		if self.specular.x != 0.0 || self.specular.y != 0.0 || self.specular.z != 0.0 {
			let light_vector = light.get_position() - contact_point;
			let distance = light_vector.dot(&light_vector).sqrt();

			let v = (eye - contact_point).normalize();
			let l = light_vector.normalize();
			let n = normal.normalize();

			let t = l.dot(&n) * 2.0;
			let r = n.map(|component| component * t) - l;
			let factor = f32::max(r.dot(&v), 0.0).powf(self.shininess);

			light
				.attenuate(distance)
				.component_mul(&self.specular)
				.component_mul(&Vector3::repeat(factor))
		} else {
			Vector3::new(0.0, 0.0, 0.0)
		}
	}
}

impl BSDF for Phong
{
	fn shade_pixel(&self, ray: &Ray, hit: &Hit, scene: &Scene) -> Vector3<f32>
	{
		let ac = self.diffuse.component_mul(&scene.get_ambient());
		let mut dc = Vector3::new(0.0, 0.0, 0.0);
		let mut sc = Vector3::new(0.0, 0.0, 0.0);

		let contact_point = ray.origin + (hit.intersect * (ray.point - ray.origin));

		for light in scene.get_lights().iter() {
			let shadow_ray = Ray::new(&contact_point, light.get_position());

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
