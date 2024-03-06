use std::f32;
use std::fs::File;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time;
use std::io::Write;

use image::png;
use image::ColorType;
use image::ImageBuffer;
use image::Pixel;
use image::Rgb;
use na::Matrix4;
use na::Vector3;
use na::Vector4;
use na::Unit;
use na::U3;
use Light;
use Object;
use Ray;
use Scene;
use shading::Material;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::util::math;

pub struct Parameters
{
	pub objects: Vec<Arc<Object>>,
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

pub fn render(parameters: Parameters)
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

	println!("{} worker threads", num_cpus::get());

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
		let mut rng = StdRng::seed_from_u64(0);

		let rgb = trace_pixel(p.0, p.1, stw, eye_4d, scene.as_ref(), &mut rng);
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
		let five_percent_pixels = total_pixels / 20;
		let mut received_pixels = 0;

		print!("\rProgress: 0%");
		std::io::stdout().flush().unwrap();

		for pixel_colour in rx {
			image.put_pixel(
				pixel_colour.x,
				pixel_colour.y,
				*Rgb::from_slice(&pixel_colour.rgb),
			);
			received_pixels += 1;

			if received_pixels % five_percent_pixels == 0{
				print!(
					"\rProgress: {:.0}%",
					(received_pixels as f32 / total_pixels as f32) * 100.0
				);
				std::io::stdout().flush().unwrap();
			}
		}

		println!("\rProgress: 100%");
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
		},
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

		let mut rng = StdRng::from_entropy();

		for x in frame_section.x..frame_section.x + frame_section.width {
			for y in frame_section.y..frame_section.y + frame_section.height {
				let rgb = trace_pixel(x, y, stw, eye, &scene, &mut rng);

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

fn get_transform_to_interaction_frame(normal: &Vector4<f32>) -> Matrix4<f32>
{
	let vertical = Vector4::new(0.0, 0.0, 1.0, 0.0);
	let nvertical = -vertical;

	let rotation_axis = if *normal == vertical || *normal == nvertical {
		Vector4::new(1.0, 0.0, 0.0, 0.0)
	} else {
		math::cross_4d(*normal, vertical)
	};

	let rotation_angle = normal.dot(&vertical).acos();
	Matrix4::from_axis_angle(&Unit::new_normalize(rotation_axis.fixed_rows::<U3>(0).into()), rotation_angle)
}

fn direct_lighting(point: Vector4<f32>, w_out: Vector4<f32>, normal: Vector4<f32>, material: &dyn Material, scene: &Scene) -> Vector3<f32>
{
	let mut l_out = Vector3::zeros();
	
	for light in scene.get_lights().iter() {
		if light.has_delta_distribution() {
			let (l_in, w_in, _pdf) = light.sample(&point, (0.0, 0.0));

			//let shadow_ray = Ray::new2(&point, &w_in);
			let shadow_ray = Ray::new(point, light.get_position());
			if let Some((shadow_hit, _)) = scene.check_hit(&shadow_ray) {
				if shadow_hit.intersect <= 1.0 {
					continue;
				}
			}

			let transform = get_transform_to_interaction_frame(&normal);

			let w_out = transform * w_out;
			let w_in = transform * w_in;

			l_out += (material.bsdf(&w_in, &w_out) * w_in.z.abs()).component_mul(&l_in);
		}
		else {
			// TODO: Non-delta lights
		}
	}

	l_out
}

fn generate_path(initial_direction: Ray, scene: &Scene, rng: &mut StdRng) -> Vector3<f32>
{
	let mut ray = initial_direction;

	let max_depth = 10;
	let mut current_depth = 0;

	let mut radiance = Vector3::zeros();
	let mut beta = Vector3::new(1.0, 1.0, 1.0);
	loop {
		if current_depth >= max_depth {
			break;
		}

		if let Some((hit, material)) = scene.check_hit(&ray) {
			let p = ray.origin() + (hit.intersect * (ray.point() - ray.origin()));
			let normal = hit.normal.normalize();
			let w_out = (ray.origin() - p).normalize();

			// At some point when emissive objects are supported we will have to conditionally account for it here
			// Consult pbrt 3rd edition for details

			let direct_illumination = direct_lighting(p, w_out, normal, material, scene);
			radiance += beta.component_mul(&direct_illumination);

			let transform = get_transform_to_interaction_frame(&normal);

			let w_out = transform * w_out;
			let (scattering, w_in, pdf) = material.sample_bsdf(&w_out, (rng.gen(), rng.gen()));

			// Degenerate case causes beta to become infinite and the pixel to become white
			if pdf == 0.0 {
				break;
			}

			let w_in = transform.try_inverse().unwrap() * w_in;
			beta.component_mul_assign(&((scattering * normal.dot(&w_in).abs()) / pdf));
			let luminance = 0.2126_f32 * beta.x + 0.7152 * beta.y + 0.0722 * beta.z;

			current_depth += 1;
			if current_depth > 3 {
				let q = 0.05_f32.max(1.0 - luminance);
				let term: f32 = rng.gen();

				if term < q {
					break;
				}

				beta /= 1.0 - q;
			}

			ray = Ray::new(p, p + w_in);
		}
		else {
			break;
		}
	}

	radiance
}

fn trace_pixel(x: u32, y: u32, stw: Matrix4<f32>, eye: Vector4<f32>, scene: &Scene, rng: &mut StdRng) -> [u8; 3]
{
	let num_samples = 1000;

	let mut radiance = Vector3::zeros();
	for _ in 0..num_samples {
		let offset_x = rng.gen::<f32>();
		let offset_y = rng.gen::<f32>();
	
		let pworld = stw * Vector4::new((x as f32) + offset_x, (y as f32) + offset_y, 0.0, 1.0);

		let ray = Ray::new(eye, pworld);
		radiance += generate_path(ray, scene, rng);
	}

	radiance /= num_samples as f32;

	let r = (255.0 * radiance[0].min(1.0)) as u8;
	let g = (255.0 * radiance[1].min(1.0)) as u8;
	let b = (255.0 * radiance[2].min(1.0)) as u8;

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

const BLOCK_SIZE: u32 = 8;
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
