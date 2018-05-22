pub use self::primitives::NonhierBox;
pub use self::primitives::NonhierSphere;
pub use self::material::Material;
pub use self::ray::Ray;
pub use self::ray::Hit;
pub use self::light::Light;
pub use self::object::Object;

pub mod traits;
pub mod math;
pub mod primitives;
pub mod material;
pub mod ray;
pub mod object;
pub mod light;

use std::sync::Arc;
use na::Vector3;

pub fn render(objects: Vec<Arc<Object>>, output_name: String, output_width: u32, output_height: u32,
              eye: Vector3<f32>, view: Vector3<f32>, up: Vector3<f32>, fov_y: f32, ambient: Vector3<f32>, lights: Vec<Arc<Light>>)
{

}

