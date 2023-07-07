use std::fmt;

use na::Vector3;
use na::Vector4;

pub trait Material: Send + Sync + fmt::Debug
{
	fn ambient_component(&self) -> Vector3<f32>;
	fn diffuse_component(&self, light: Vector4<f32>, normal: Vector4<f32>) -> Vector3<f32>;
	fn specular_component(
		&self,
		view: Vector4<f32>,
		light: Vector4<f32>,
		normal: Vector4<f32>,
	) -> Vector3<f32>;

	#[allow(unused)]
	fn emissive_component(&self, origin: Vector4<f32>, contact: Vector4<f32>) -> Vector3<f32>
	{
		Vector3::zeros()
	}
}
