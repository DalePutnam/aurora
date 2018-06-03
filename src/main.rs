extern crate rlua;
extern crate image;
extern crate nalgebra as na;
extern crate num_cpus;

mod core;
mod scene_builder;

use scene_builder::SceneBuilder;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("No input file specified, exiting.");
    } else {
        let input_file = &args[1];
        let scene_builder = SceneBuilder::new();

        match scene_builder.run_build_script(input_file) {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        };
    }
}
