extern crate clap;
extern crate failure;
extern crate image;
extern crate nalgebra as na;
extern crate num_cpus;
extern crate rand;
extern crate rlua;
extern crate thread_local;

pub use self::grid::Grid;
pub use self::light::Light;
pub use self::object::Object;
pub use self::ray::Hit;
pub use self::ray::Ray;
pub use self::scene::Scene;

pub mod cli;
pub mod grid;
pub mod light;
pub mod lua;
pub mod object;
pub mod primitives;
pub mod ray;
pub mod render;
pub mod scene;
pub mod shading;
pub mod util;

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
