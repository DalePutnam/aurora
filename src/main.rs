extern crate failure;
extern crate image;
extern crate nalgebra as na;
extern crate num_cpus;
extern crate rand;
extern crate rlua;
extern crate thread_local;
extern crate clap;

pub use self::bounding_box::BoundingBox;
pub use self::cook_torrance::CookTorrance;
pub use self::grid::Grid;
pub use self::light::Light;
pub use self::material::Material;
pub use self::mesh::Mesh;
pub use self::object::Object;
pub use self::phong::Phong;
pub use self::primitives::Cube;
pub use self::primitives::NonhierBox;
pub use self::primitives::NonhierSphere;
pub use self::primitives::Sphere;
pub use self::ray::Hit;
pub use self::ray::Ray;
pub use self::scene::Scene;

pub mod bounding_box;
pub mod cook_torrance;
pub mod grid;
pub mod light;
pub mod lua;
pub mod material;
pub mod mesh;
pub mod object;
pub mod phong;
pub mod primitives;
pub mod ray;
pub mod render;
pub mod scene;
pub mod traits;
pub mod util;
pub mod cli;

use lua::SceneBuilder;

fn main()
{
	let parameters = cli::parse_args();

	let input_file = parameters.input_file.clone();

	let scene_builder = SceneBuilder::new(parameters);

	match scene_builder.run_build_script(&input_file) {
		Ok(_) => (),
		Err(e) => println!("{}", e),
	};
}
