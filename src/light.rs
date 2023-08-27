use na::Vector3;
use na::Vector4;

use std::f32;

#[derive(Clone)]
pub struct Light
{
	position: Vector4<f32>,
	colour: Vector3<f32>,
	radiant_intensity: f32,
	falloff: Vector3<f32>,
}

impl Light
{
	pub fn new(position: Vector3<f32>, colour: Vector3<f32>, falloff: Vector3<f32>) -> Self
	{
		Light {
			position: Vector4::<f32>::new(position.x, position.y, position.z, 1.0),
			colour: colour,
			radiant_intensity: 1.0  / (f32::consts::PI * 4.0),
			falloff: falloff,
		}
	}

	pub fn new2(position: Vector3<f32>, colour: Vector3<f32>, power: f32) -> Self
	{
		Light {
			position: Vector4::<f32>::new(position.x, position.y, position.z, 1.0),
			colour: colour,
			radiant_intensity: power / (f32::consts::PI * 4.0),
			falloff: Vector3::new(1.0, 0.0, 0.0),
		}
	}

	pub fn sample(&self, point: &Vector4<f32>, u: (f32, f32)) -> (Vector3<f32>, Vector4<f32>, f32)
	{
		let w = self.position - point;

		let distance = w.magnitude();
		let radiance = self.colour * (self.radiant_intensity / (distance * distance));

		(radiance, w.normalize(), 1.0)
	}

	pub fn pdf(&self, _point: &Vector4<f32>, _w_in: &Vector4<f32>) -> f32
	{
		0.0
	}

	pub fn has_delta_distribution(&self) -> bool
	{
		true
	}

	pub fn get_position(&self) -> Vector4<f32>
	{
		self.position
	}

	pub fn get_colour(&self) -> Vector3<f32>
	{
		self.colour
	}

	pub fn attenuate(&self, distance: f32) -> Vector3<f32>
	{
		self.colour
			/ (self.falloff.x
				+ (self.falloff.y * distance)
				+ (self.falloff.z * distance * distance))
	}
}
