use std::fmt;

use na::Vector3;
use na::Vector4;
use shading::Material;

use util::sampling;

use std::f32;

#[derive(fmt::Debug)]
pub struct Lambertian
{
	colour: Vector3<f32>,
}

impl Lambertian
{
	pub fn new(colour: Vector3<f32>) -> Self
	{
		Lambertian {
			colour: colour,
		}
	}
}
 
impl Material for Lambertian
{
	fn bsdf(&self, _w_in: &Vector4<f32>, _w_out: &Vector4<f32>) -> Vector3<f32>
	{
		self.colour / f32::consts::PI
	}

	fn sample_bsdf(&self, w_out: &Vector4<f32>, u: (f32, f32)) -> (Vector3<f32>, Vector4<f32>, f32)
	{
		let w_in = sampling::cosine_sample_hemisphere(u);
		let pdf = self.pdf(&w_in);
		let bdsf = self.bsdf(&w_in, &w_out);

		(bdsf, w_in, pdf)
	}

	fn pdf(&self, w_in: &Vector4<f32>) -> f32
	{
		sampling::cosine_hemisphere_pdf(w_in.z.abs())
	}

	fn ambient_component(&self) -> Vector3<f32>
	{
		self.colour
	}

	fn diffuse_component(&self, light: Vector4<f32>, normal: Vector4<f32>) -> Vector3<f32>
	{
		self.colour * light.dot(&normal).max(0.0)
	}

	fn specular_component(
		&self,
		_view: Vector4<f32>,
		_light: Vector4<f32>,
		_normal: Vector4<f32>,
	) -> Vector3<f32>
	{
		Vector3::zeros()
	}
}
