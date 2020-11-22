extern crate failure;
extern crate image;
extern crate nalgebra as na;
extern crate num_cpus;
extern crate rlua;

pub use self::bounding_box::BoundingBox;
pub use self::light::Light;
pub use self::material::Material;
pub use self::mesh::Mesh;
pub use self::object::Object;
pub use self::phong::Phong;
pub use self::primitives::Cube;
pub use self::primitives::NonhierBox;
pub use self::primitives::NonhierSphere;
pub use self::primitives::PrimitivePtr;
pub use self::primitives::Sphere;
pub use self::ray::Hit;
pub use self::ray::Ray;
pub use self::scene::Scene;

pub mod bounding_box;
pub mod light;
pub mod lua;
pub mod material;
pub mod mesh;
pub mod object;
pub mod phong;
pub mod primitives;
pub mod ray;
pub mod scene;
pub mod traits;
pub mod util;

use image::{png, ColorType, ImageBuffer, Pixel, Rgb};
use lua::SceneBuilder;
use na::{Matrix4, Vector3, Vector4};
use std::f32;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn render(
    objects: Vec<Object>,
    output_name: String,
    output_width: u32,
    output_height: u32,
    eye: Vector3<f32>,
    view: Vector3<f32>,
    up: Vector3<f32>,
    fov_y: f32,
    ambient: Vector3<f32>,
    lights: Vec<Light>,
) {
    println!("Aurora Ray Tracer");
    println!("Rendering to {}", output_name);
    println!("Width: {} Height: {}", output_width, output_height);
    println!("Vertical FOV: {}", fov_y);
    println!("Eye:  {{ x: {}, y: {}, z: {} }}", eye.x, eye.y, eye.z);
    println!("View: {{ x: {}, y: {}, z: {} }}", view.x, view.y, view.z);
    println!("Up:   {{ x: {}, y: {}, z: {} }}", up.x, up.y, up.z);
    println!(
        "Rendering {} objects with {} lights",
        objects.len(),
        lights.len()
    );

    let stw = create_screen_to_world_matrix(output_width, output_height, fov_y, eye, view, up);
    let eye_4d = Vector4::new(eye.x, eye.y, eye.z, 1.0);

    let frame_sections = Arc::new(Mutex::new(divide_frame(output_width, output_height)));
    let cpus = num_cpus::get();

    let scene = Arc::new(Scene::new(objects, lights, ambient));

    let rx = {
        let (tx, rx) = mpsc::channel();

        for cpu in 0..cpus {
            let frame_sections = Arc::clone(&frame_sections);
            let tx = mpsc::Sender::clone(&tx);
            let scene = Arc::clone(&scene);

            thread::spawn(move || {
                trace_worker(cpu, stw, eye_4d, scene.as_ref(), frame_sections, tx);
            });
        }

        rx
    };

    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(output_width, output_height);
    let total_pixels = output_width * output_height;
    let mut received_pixels = 0;

    for pixel_colour in rx {
        image.put_pixel(
            pixel_colour.x,
            pixel_colour.y,
            *Rgb::from_slice(&pixel_colour.rgb),
        );
        received_pixels += 1;

        if received_pixels % 1000 == 0 {
            print!(
                "Progress: {:.0}%\r",
                (received_pixels as f32 / total_pixels as f32) * 100.0
            );
        }
    }

    println!("Progress: 100%");
    println!("Done");

    match File::create(&output_name) {
        Ok(file) => {
            let encoder = png::PNGEncoder::new(file);
            match encoder.encode(
                &image.into_raw(),
                output_width,
                output_height,
                ColorType::RGB(8),
            ) {
                Ok(_) => (),
                Err(e) => println!("ERROR: Unable to encode image: {}", e),
            }
        }
        Err(e) => println!("ERROR: Unable to write to file {}: {}", output_name, e),
    }
}

fn trace_worker(
    cpu: usize,
    stw: Matrix4<f32>,
    eye: Vector4<f32>,
    scene: &Scene,
    frame_sections: Arc<Mutex<Vec<FrameSection>>>,
    tx: Sender<PixelColour>,
) {
    loop {
        let frame_section = match frame_sections.lock() {
            Ok(mut sections) => match sections.pop() {
                Some(section) => section,
                None => break,
            },
            Err(_) => {
                println!("Frame section lock is poisoned! Thread {} exiting.", cpu);
                break;
            }
        };

        for x in frame_section.x..frame_section.x + frame_section.width {
            for y in frame_section.y..frame_section.y + frame_section.height {
                let pworld = stw * Vector4::new(x as f32, y as f32, 0.0, 1.0);
                let ray = Ray {
                    point: pworld,
                    origin: eye,
                    id: 0,
                    thread_id: 0,
                };

                let colour = trace_pixel(&ray, &scene);

                let r = (255.0 * colour.x.min(1.0)) as u8;
                let g = (255.0 * colour.y.min(1.0)) as u8;
                let b = (255.0 * colour.z.min(1.0)) as u8;

                if let Err(_) = tx.send(PixelColour {
                    x: x,
                    y: y,
                    rgb: [r, g, b],
                }) {
                    println!("Receiver closed unexpectedly! Thread {} exiting.", cpu);
                    break;
                }
            }
        }
    }
}

fn trace_pixel(ray: &Ray, scene: &Scene) -> Vector3<f32> {
    match scene.check_hit(ray) {
        Some((hit, material)) => material.shade_pixel(&ray, &hit, scene),
        None => Vector3::new(0.0, 0.0, 0.0),
    }
}

fn create_screen_to_world_matrix(
    width: u32,
    height: u32,
    fov_y: f32,
    eye: Vector3<f32>,
    view: Vector3<f32>,
    up: Vector3<f32>,
) -> Matrix4<f32> {
    let nx = width as f32;
    let ny = height as f32;

    let hi = 2.0;
    let wi = (nx * hi) / ny;
    let d = hi / (2.0 * (fov_y / 2.0).to_radians().tan());

    let w = (view - eye).normalize();
    let u = up.cross(&w).normalize();
    let v = u.cross(&w);

    let t1 = Matrix4::new_translation(&Vector3::new(-nx / 2.0, -ny / 2.0, d));
    let s2 = Matrix4::new_nonuniform_scaling(&Vector3::new(-hi / ny, wi / nx, 1.0));
    let r3 = Matrix4::new(
        u.x, v.x, w.x, 0.0, u.y, v.y, w.y, 0.0, u.z, v.z, w.z, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    let t4 = Matrix4::new(
        1.0, 0.0, 0.0, eye.x, 0.0, 1.0, 0.0, eye.y, 0.0, 0.0, 1.0, eye.z, 0.0, 0.0, 0.0, 1.0,
    );

    t4 * r3 * s2 * t1
}

const BLOCK_SIZE: u32 = 64;
fn divide_frame(width: u32, height: u32) -> Vec<FrameSection> {
    let cols = width / BLOCK_SIZE;
    let rows = height / BLOCK_SIZE;
    let col_remainder = width % BLOCK_SIZE;
    let row_remainder = height % BLOCK_SIZE;

    let mut sections = Vec::new();
    for y in 0..rows {
        for x in 0..cols {
            sections.push(FrameSection {
                x: x * BLOCK_SIZE,
                y: y * BLOCK_SIZE,
                width: BLOCK_SIZE,
                height: BLOCK_SIZE,
            });
        }
    }

    if col_remainder > 0 {
        // Right edge remainder
        for y in 0..rows {
            sections.push(FrameSection {
                x: cols * BLOCK_SIZE,
                y: y * BLOCK_SIZE,
                width: col_remainder,
                height: BLOCK_SIZE,
            });
        }
    }

    if row_remainder > 0 {
        // Bottom edge remainder
        for x in 0..cols {
            sections.push(FrameSection {
                x: x * BLOCK_SIZE,
                y: rows * BLOCK_SIZE,
                width: BLOCK_SIZE,
                height: row_remainder,
            });
        }
    }

    if col_remainder > 0 && row_remainder > 0 {
        // Bottom right corner
        sections.push(FrameSection {
            x: cols * BLOCK_SIZE,
            y: rows * BLOCK_SIZE,
            width: col_remainder,
            height: row_remainder,
        });
    }

    sections
}

struct FrameSection {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

struct PixelColour {
    pub x: u32,
    pub y: u32,
    pub rgb: [u8; 3],
}

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
