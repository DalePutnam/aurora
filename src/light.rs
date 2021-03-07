use na::Vector3;
use na::Vector4;

#[derive(Clone)]
pub struct Light
{
	position: Vector4<f32>,
	colour: Vector3<f32>,
	falloff: Vector3<f32>,
}

impl Light
{
	pub fn new(position: &Vector3<f32>, colour: &Vector3<f32>, falloff: &Vector3<f32>) -> Self
	{
		Light {
			position: Vector4::<f32>::new(position.x, position.y, position.z, 1.0),
			colour: *colour,
			falloff: *falloff,
		}
	}

	pub fn get_position(&self) -> &Vector4<f32>
	{
		&self.position
	}

	pub fn get_colour(&self) -> &Vector3<f32>
	{
		&self.colour
	}

	pub fn attenuate(&self, distance: f32) -> Vector3<f32>
	{
		self.colour
			/ (self.falloff.x
				+ (self.falloff.y * distance)
				+ (self.falloff.z * distance * distance))
	}
}
