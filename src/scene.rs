use na::Vector3;
use Grid;
use Hit;
use Light;
use Material;
use Object;
use Ray;

pub struct Scene
{
	grid: Grid,
	lights: Vec<Light>,
	ambient: Vector3<f32>,
}

impl Scene
{
	pub fn new(objects: Vec<Object>, lights: Vec<Light>, ambient: Vector3<f32>) -> Self
	{
		Scene {
			grid: Grid::new(objects),
			lights: lights,
			ambient: ambient,
		}
	}

	pub fn check_hit(&self, ray: &Ray) -> Option<(Hit, &Material)>
	{
		self.grid.check_hit(ray)
	}

	pub fn get_lights(&self) -> &Vec<Light>
	{
		&self.lights
	}

	pub fn get_ambient(&self) -> Vector3<f32>
	{
		self.ambient
	}
}
