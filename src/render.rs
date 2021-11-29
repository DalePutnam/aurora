use std::f32;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use image::png;
use image::ColorType;
use image::ImageBuffer;
use image::Pixel;
use image::Rgb;
use na::Matrix4;
use na::Vector3;
use na::Vector4;
use Light;
use Object;
use Ray;
use Scene;

pub struct Parameters
{
	pub objects: Vec<Object>,
	pub lights: Vec<Light>,
	pub output_file: String,
	pub resolution: (u32, u32),
	pub eye_vector: Vector3<f32>,
	pub view_vector: Vector3<f32>,
	pub up_vector: Vector3<f32>,
	pub vertical_fov: f32,
	pub ambient_light: Vector3<f32>,
	pub single_pixel: Option<(u32, u32)>,
}

pub fn render(
	parameters: Parameters,
)
{
	let output_file = parameters.output_file;
	let image_width = parameters.resolution.0;
	let image_height = parameters.resolution.1;
	let vertical_fov = parameters.vertical_fov;
	let eye_vector = parameters.eye_vector;
	let view_vector = parameters.view_vector;
	let up_vector = parameters.up_vector;

	println!("Aurora Ray Tracer");
	println!("Rendering to {}", output_file);
	println!("Width: {} Height: {}", image_width, image_height);
	println!("Vertical FOV: {}", vertical_fov);
	println!(
		"Eye:  {{ x: {}, y: {}, z: {} }}",
		eye_vector.x, eye_vector.y, eye_vector.z
	);
	println!(
		"View: {{ x: {}, y: {}, z: {} }}",
		view_vector.x, view_vector.y, view_vector.z
	);
	println!(
		"Up:   {{ x: {}, y: {}, z: {} }}",
		up_vector.x, up_vector.y, up_vector.z
	);

	if let Some(p) = &parameters.single_pixel {
		println!("Rendering single pixel: x: {} y: {}", p.0, p.1);
	}

	let objects = parameters.objects;
	let lights = parameters.lights;
	let ambient_light = parameters.ambient_light;

	println!(
		"Rendering {} objects with {} lights",
		objects.len(),
		lights.len()
	);

	let stw = create_screen_to_world_matrix(
		image_width,
		image_height,
		vertical_fov,
		eye_vector,
		view_vector,
		up_vector,
	);
	let eye_4d = Vector4::new(eye_vector.x, eye_vector.y, eye_vector.z, 1.0);

	let frame_sections = Arc::new(Mutex::new(divide_frame(image_width, image_height)));

	let scene = Arc::new(Scene::new(objects, lights, ambient_light));

	let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(
		image_width,
		image_height,
		*Rgb::from_slice(&[0, 0, 0]),
	);

	if let Some(p) = &parameters.single_pixel {
		let rgb = trace_pixel(p.0, p.1, stw, eye_4d, scene.as_ref());
		image.put_pixel(p.0, p.1, *Rgb::from_slice(&rgb));
	} else {
		let rx = {
			let (tx, rx) = mpsc::channel();

			for _ in 0..num_cpus::get() {
				let frame_sections = Arc::clone(&frame_sections);
				let tx = mpsc::Sender::clone(&tx);
				let scene = Arc::clone(&scene);

				thread::spawn(move || {
					trace_worker(stw, eye_4d, scene.as_ref(), frame_sections, tx);
				});
			}

			rx
		};

		let total_pixels = image_width * image_height;
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
	}

	match File::create(&output_file) {
		Ok(file) => {
			let encoder = png::PNGEncoder::new(file);
			match encoder.encode(
				&image.into_raw(),
				image_width,
				image_height,
				ColorType::RGB(8),
			) {
				Ok(_) => (),
				Err(e) => println!("ERROR: Unable to encode image: {}", e),
			}
		}
		Err(e) => println!("ERROR: Unable to write to file {}: {}", output_file, e),
	}
}

fn trace_worker(
	stw: Matrix4<f32>,
	eye: Vector4<f32>,
	scene: &Scene,
	frame_sections: Arc<Mutex<Vec<FrameSection>>>,
	tx: Sender<PixelColour>,
)
{
	loop {
		let frame_section = {
			let mut frame_sections = frame_sections.lock().unwrap();

			match frame_sections.pop() {
				Some(frame_section) => frame_section,
				None => break,
			}
		};

		for x in frame_section.x..frame_section.x + frame_section.width {
			for y in frame_section.y..frame_section.y + frame_section.height {
				let rgb = trace_pixel(x, y, stw, eye, &scene);

				tx.send(PixelColour {
					x: x,
					y: y,
					rgb: rgb,
				})
				.unwrap();
			}
		}
	}
}

fn trace_pixel(x: u32, y: u32, stw: Matrix4<f32>, eye: Vector4<f32>, scene: &Scene) -> [u8; 3]
{
	let pworld = stw * Vector4::new(x as f32, y as f32, 0.0, 1.0);
	let ray = Ray::new(eye, pworld);

	let colour_vec = match scene.check_hit(&ray) {
		Some((hit, material)) => {
			let contact_point = ray.origin() + (hit.intersect * (ray.point() - ray.origin()));
			let view_vector = (eye - contact_point).normalize();
			let normal = hit.normal.normalize();

			let ac = scene.get_ambient().component_mul(&material.ambient_component());
			let mut dc = Vector3::new(0.0, 0.0, 0.0);
			let mut sc = Vector3::new(0.0, 0.0, 0.0);

			for light in scene.get_lights().iter() {
				let shadow_ray = Ray::new(contact_point, light.get_position());

				if let Some((shadow_hit, _)) = scene.check_hit(&shadow_ray) {
					if shadow_hit.intersect <= 1.0 {
						continue;
					}
				}

				let light_vector = light.get_position() - contact_point;
				let distance = light_vector.dot(&light_vector).sqrt();

				let light_vector = light_vector.normalize();

				sc += light.attenuate(distance).component_mul(&material.specular_component(view_vector, light_vector, normal));
				dc += light.attenuate(distance).component_mul(&material.diffuse_component(light_vector, normal));
			}

			ac + dc + sc
		},
		None => Vector3::new(0.0, 0.0, 0.0),
	};

	let r = (255.0 * colour_vec[0].min(1.0)) as u8;
	let g = (255.0 * colour_vec[1].min(1.0)) as u8;
	let b = (255.0 * colour_vec[2].min(1.0)) as u8;

	[r, g, b]
}

fn create_screen_to_world_matrix(
	width: u32,
	height: u32,
	fov_y: f32,
	eye: Vector3<f32>,
	view: Vector3<f32>,
	up: Vector3<f32>,
) -> Matrix4<f32>
{
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
fn divide_frame(width: u32, height: u32) -> Vec<FrameSection>
{
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

struct FrameSection
{
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32,
}

struct PixelColour
{
	pub x: u32,
	pub y: u32,
	pub rgb: [u8; 3],
}
