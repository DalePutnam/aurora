extern crate rlua;
extern crate nalgebra as na;

mod core;
mod lua;

use std::fs::File;
use std::io::Read;
use lua::scene_lua;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("No input file specified, exiting.");
    } else {
        let input_file = &args[1];
        let mut file = match File::open(input_file) {
            Ok(f) => f,
            Err(e) => {
                println!("Failed to open file {}: {}", input_file, e);
                return;
            },
        };

        let mut contents = String::new();

        match file.read_to_string(&mut contents) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to read {} contents: {}", input_file, e);
                return;
            }
        };

        let lua = scene_lua::initialize_lua();
        match lua.exec::<()>(&contents, Some(input_file)) {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to execute script {}: {}", input_file, e);
                return;
            },
        };
    }
}
