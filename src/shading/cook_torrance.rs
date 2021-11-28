use std::f32;
use std::fmt;

use na::Vector3;
use na::Vector4;
use shading::BSDF;
use util::math;
use Hit;
use Light;
use Ray;
use Scene;

#[derive(fmt::Debug)]
pub struct CookTorrance
{
	diffuse_colour: Vector3<f32>,
	specular_colour: Vector3<f32>,
	diffuse_fraction: f32,
	roughness: f32,
	refractive_index: f32,
	extinction_coefficient: f32,
}

impl CookTorrance
{
	pub fn new(
		diffuse_colour: Vector3<f32>,
		specular_colour: Vector3<f32>,
		diffuse_fraction: f32,
		roughness: f32,
		refractive_index: f32,
		extinction_coefficient: f32,
	) -> Self
	{
		CookTorrance {
			diffuse_colour: diffuse_colour,
			specular_colour: specular_colour,
			diffuse_fraction: diffuse_fraction,
			roughness: roughness,
			refractive_index: refractive_index,
			extinction_coefficient: extinction_coefficient,
		}
	}

	fn calculate_diffuse(
		&self,
		contact_point: Vector4<f32>,
		normal: Vector4<f32>,
		light: &Light,
	) -> Vector3<f32>
	{
		if math::near_zero(self.diffuse_fraction) {
			return Vector3::new(0.0, 0.0, 0.0);
		}

		let light_vector = light.get_position() - contact_point;
		let distance = light_vector.dot(&light_vector).sqrt();
		let diffuse_fraction = f32::max(light_vector.normalize().dot(&normal.normalize()), 0.0)
			* self.diffuse_fraction;

		light
			.attenuate(distance)
			.component_mul(&self.diffuse_colour)
			* diffuse_fraction
	}

	fn calculate_specular(
		&self,
		contact_point: Vector4<f32>,
		eye: Vector4<f32>,
		normal: Vector4<f32>,
		light: &Light,
	) -> Vector3<f32>
	{
		let specular_fraction = 1.0 - self.diffuse_fraction;

		if math::near_zero(specular_fraction) {
			return Vector3::new(0.0, 0.0, 0.0);
		}

		let light_vector = light.get_position() - contact_point;
		let distance = light_vector.dot(&light_vector).sqrt();

		let v = (eye - contact_point).normalize();
		let l = light_vector.normalize();
		let n = normal.normalize();
		let h = (v + l).normalize();

		let nv = n.dot(&v);

		if math::near_zero(nv) {
			return Vector3::new(0.0, 0.0, 0.0);
		}

		let fresnel_n =
			fresnel_from_refractive_index(1.0, self.refractive_index, self.extinction_coefficient);

		let fresnel_vh = fresnel_from_refractive_index(
			v.dot(&h),
			self.refractive_index,
			self.extinction_coefficient,
		);

		let fresnel_red = fresnel_approximation(fresnel_vh, fresnel_n, self.specular_colour.x);
		let fresnel_green = fresnel_approximation(fresnel_vh, fresnel_n, self.specular_colour.y);
		let fresnel_blue = fresnel_approximation(fresnel_vh, fresnel_n, self.specular_colour.z);

		let d = ggx_distribution(h, n, self.roughness);
		let g = ggx_geometry(v, l, h, n, self.roughness);

		let specular_partial = (d * g) / (4.0 * nv);

		let specular_colour = Vector3::new(
			fresnel_red * specular_partial,
			fresnel_green * specular_partial,
			fresnel_blue * specular_partial,
		);

		light.attenuate(distance).component_mul(&specular_colour) * specular_fraction
	}
}

impl BSDF for CookTorrance
{
	fn shade_pixel(&self, ray: &Ray, hit: &Hit, scene: &Scene) -> Vector3<f32>
	{
		let ac = self.diffuse_colour.component_mul(&scene.get_ambient()) * f32::consts::PI;
		let mut dc = Vector3::new(0.0, 0.0, 0.0);
		let mut sc = Vector3::new(0.0, 0.0, 0.0);

		let contact_point = ray.origin() + (hit.intersect * (ray.point() - ray.origin()));

		for light in scene.get_lights().iter() {
			let shadow_ray = Ray::new(contact_point, light.get_position());

			if let Some((shadow_hit, _)) = scene.check_hit(&shadow_ray) {
				if shadow_hit.intersect <= 1.0 {
					continue;
				}
			}

			dc += self.calculate_diffuse(contact_point, hit.normal, &light);
			sc += self.calculate_specular(contact_point, ray.origin(), hit.normal, &light);
		}

		ac + dc + sc
	}
}

fn chi(a: f32) -> f32
{
	if a > 0.0 {
		1.0
	} else {
		0.0
	}
}

fn ggx_distribution(half: Vector4<f32>, normal: Vector4<f32>, alpha: f32) -> f32
{
	let a2 = alpha.powi(2);
	let hn = half.dot(&normal);
	let hn2 = hn.powi(2);

	(chi(hn) * a2) / (f32::consts::PI * ((hn2 * a2) + (1.0 - hn2)).powi(2))
}

fn ggx_geometry(
	view: Vector4<f32>,
	light: Vector4<f32>,
	half: Vector4<f32>,
	normal: Vector4<f32>,
	alpha: f32,
) -> f32
{
	ggx_geometry_partial(view, half, normal, alpha)
		* ggx_geometry_partial(light, half, normal, alpha)
}

fn ggx_geometry_partial(
	direction: Vector4<f32>,
	half: Vector4<f32>,
	normal: Vector4<f32>,
	alpha: f32,
) -> f32
{
	let a2 = alpha.powi(2);
	let dh = direction.dot(&half);
	let dn = direction.dot(&normal);
	let dn2 = dn.powi(2);
	let tan2 = (1.0 - dn2) / dn2;

	(chi(dh / dn) * 2.0) / (1.0 + (1.0 + (a2 * tan2)).sqrt())
}

fn fresnel_from_refractive_index(
	cosine_angle: f32,
	refractive_index: f32,
	extinction_coefficient: f32,
) -> f32
{
	let u = cosine_angle;
	let n = refractive_index;
	let k = extinction_coefficient;

	let k2 = k * k;

	((n - 1.0).powi(2) + 4.0 * n * (1.0 - u).powi(5) + k2) / ((n + 1.0).powi(2) + k2)
}

fn fresnel_approximation(fresnel: f32, fresnel_normal: f32, reflectance: f32) -> f32
{
	let r = reflectance;
	let nf = fresnel_normal;
	let f = fresnel;

	r + (1.0 - r) * ((f - nf) / (1.0 - nf))
}
