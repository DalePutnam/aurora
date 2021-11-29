use std::f32;
use std::fmt;

use na::Vector3;
use na::Vector4;
use shading::Material;
use util::math;

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
}

impl Material for CookTorrance
{
	fn ambient_component(&self) -> Vector3<f32> 
	{
		self.diffuse_colour * f32::consts::PI
	}

	fn diffuse_component(
		&self,
		light: Vector4<f32>,
		normal: Vector4<f32>
	) -> Vector3<f32>
	{
		if math::near_zero(self.diffuse_fraction) {
			return Vector3::new(0.0, 0.0, 0.0);
		}

		self.diffuse_colour * f32::max(light.dot(&normal), 0.0) * self.diffuse_fraction
	}

	fn specular_component(
		&self,
		view: Vector4<f32>,
		light: Vector4<f32>,
		normal: Vector4<f32>
	) -> Vector3<f32>
	{
		let specular_fraction = 1.0 - self.diffuse_fraction;

		if math::near_zero(specular_fraction) {
			return Vector3::new(0.0, 0.0, 0.0);
		}

		let half = (view + light).normalize();

		let nv = normal.dot(&view);

		if math::near_zero(nv) {
			return Vector3::new(0.0, 0.0, 0.0);
		}

		let fresnel_n =
			fresnel_from_refractive_index(1.0, self.refractive_index, self.extinction_coefficient);

		let fresnel_vh = fresnel_from_refractive_index(
			view.dot(&half),
			self.refractive_index,
			self.extinction_coefficient,
		);

		let fresnel_red = fresnel_approximation(fresnel_vh, fresnel_n, self.specular_colour.x);
		let fresnel_green = fresnel_approximation(fresnel_vh, fresnel_n, self.specular_colour.y);
		let fresnel_blue = fresnel_approximation(fresnel_vh, fresnel_n, self.specular_colour.z);

		let d = ggx_distribution(half, normal, self.roughness);
		let g = ggx_geometry(view, light, half, normal, self.roughness);

		let specular_partial = (d * g) / (4.0 * nv);

		let specular_colour = Vector3::new(
			fresnel_red * specular_partial,
			fresnel_green * specular_partial,
			fresnel_blue * specular_partial,
		);

		specular_colour * specular_fraction
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
