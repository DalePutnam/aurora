use std::fmt;

use na::Vector3;
use na::Vector4;
use shading::Material;

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
