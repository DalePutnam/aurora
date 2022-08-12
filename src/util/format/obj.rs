use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;

use na::Vector4;

pub const FILE_EXTENSION: &str = "obj";

pub struct Mesh
{
	pub v: Vec<Vector4<f32>>,
	pub vn: Vec<Vector4<f32>>,
	pub vt: Vec<Vector4<f32>>,
	pub f: Vec<Face>,
}

pub struct Face
{
	pub v: (usize, usize, usize),
	pub vn: Option<(usize, usize, usize)>,
	pub vt: Option<(usize, usize, usize)>,
}

#[derive(fmt::Debug)]
pub enum Error
{
	ParseError
	{
		file_name: String,
		line_number: usize,
		message: String,
	},
	IOError
	{
		file_name: String,
		error: io::Error,
	},
}

impl fmt::Display for Error
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>
	{
		match self {
			Self::ParseError {
				file_name,
				line_number,
				message,
			} => {
				return write!(f, "{}:{} error: {}", file_name, line_number, message);
			},
			Self::IOError { file_name, error } => {
				return write!(f, "Failed to read {}: {}", file_name, error);
			},
		}
	}
}

impl error::Error for Error {}

impl Error
{
	fn parse_error(file_name: String, line_number: usize, message: String) -> Self
	{
		Self::ParseError {
			file_name: file_name,
			line_number: line_number,
			message: message,
		}
	}

	fn io_error(file_name: String, error: io::Error) -> Self
	{
		Self::IOError {
			file_name: file_name,
			error: error,
		}
	}
}

pub fn read_file(path: &str) -> Result<Mesh, Error>
{
	let obj_file = match File::open(path) {
		Ok(file) => file,
		Err(error) => return Err(Error::io_error(String::from(path), error)),
	};

	let reader = BufReader::new(obj_file);

	let mut mesh = Mesh {
		v: Vec::new(),
		vn: Vec::new(),
		vt: Vec::new(),
		f: Vec::new(),
	};

	let mut line_number = 1;

	for line in reader.lines() {
		let line = match line {
			Ok(line) => line,
			Err(error) => return Err(Error::io_error(String::from(path), error)),
		};

		if let Err(message) = parse_line(line, &mut mesh) {
			return Err(Error::parse_error(String::from(path), line_number, message));
		}

		line_number += 1;
	}

	Ok(mesh)
}

fn parse_line(line: String, mesh: &mut Mesh) -> Result<(), String>
{
	let line = line.trim();

	match line.chars().nth(0) {
		Some(char) => {
			if char == '#' {
				return Ok(());
			}
		},
		None => return Ok(()),
	};

	let line_parts: Vec<&str> = line.split_whitespace().collect();

	if line_parts.len() >= 1 {
		match line_parts[0] {
			"v" => {
				let vertex = parse_vertex_data(&line_parts)?;
				mesh.v.push(vertex);
			},
			"vn" => {
				let normal = parse_vertex_data(&line_parts)?;
				mesh.vn.push(normal);
			},
			"vt" => {
				let texture_coordinate = parse_vertex_data(&line_parts)?;
				mesh.vt.push(texture_coordinate);
			},
			"f" => {
				let face = parse_face_data(&line_parts)?;
				mesh.f.push(face);
			},
			"o" | "g" | "s" => {
				// Object names (o), groups (g), smoothing groups (s) are ignored
			},
			_ => return Err(format!("\"{}\" tag not supported", line_parts[0])),
		};
	}

	Ok(())
}

fn parse_vertex_data(parts: &Vec<&str>) -> Result<Vector4<f32>, String>
{
	if parts.len() < 4 {
		return Err(String::from(
			"Vertices with fewer than 3 coordinates are not supported",
		));
	} else if parts.len() > 4 {
		return Err(String::from(
			"Vertices with more than 3 coordinates are not supported",
		));
	}

	let x = parse_vertex_component(parts[1])?;
	let y = parse_vertex_component(parts[2])?;
	let z = parse_vertex_component(parts[3])?;

	Ok(Vector4::new(x, y, z, 1.0))
}

fn parse_vertex_component(part: &str) -> Result<f32, String>
{
	match part.parse::<f32>() {
		Ok(value) => Ok(value),
		Err(_) => Err(format!("Failed to parse vertex data \"{}\"", part)),
	}
}

fn parse_face_data(parts: &Vec<&str>) -> Result<Face, String>
{
	if parts.len() < 4 {
		return Err(String::from(
			"Faces with fewer than 3 vertices are not supported",
		));
	} else if parts.len() > 4 {
		return Err(String::from(
			"Faces with more than 3 vertices are not supported",
		));
	}

	let v1_data = parse_face_component(parts[1])?;
	let v2_data = parse_face_component(parts[2])?;
	let v3_data = parse_face_component(parts[3])?;

	let vertices = (v1_data.0, v2_data.0, v3_data.0);

	let normals = match (v1_data.1, v2_data.1, v3_data.1) {
		(Some(vn1), Some(vn2), Some(vn3)) => Some((vn1, vn2, vn3)),
		(None, None, None) => None,
		_ => return Err(String::from("Some vertices missing normal data")),
	};

	let texture_coordinates = match (v1_data.2, v2_data.2, v3_data.2) {
		(Some(vt1), Some(vt2), Some(vt3)) => Some((vt1, vt2, vt3)),
		(None, None, None) => None,
		_ => {
			return Err(String::from(
				"Some vertices missing texture coordinate data",
			))
		},
	};

	Ok(Face {
		v: vertices,
		vn: normals,
		vt: texture_coordinates,
	})
}

fn parse_face_component(component: &str) -> Result<(usize, Option<usize>, Option<usize>), String>
{
	let parts: Vec<&str> = component.split('/').collect();

	if parts.len() == 1 {
		let vertex_index = parse_face_component_index(parts[0])?;

		Ok((vertex_index - 1, None, None))
	} else if parts.len() == 2 {
		let vertex_index = parse_face_component_index(parts[0])?;
		let texture_coordinate_index = parse_face_component_index(parts[1])?;

		Ok((vertex_index - 1, None, Some(texture_coordinate_index - 1)))
	} else if parts.len() == 3 {
		if parts[1].is_empty() {
			let vertex_index = parse_face_component_index(parts[0])?;
			let normal_index = parse_face_component_index(parts[2])?;

			Ok((vertex_index - 1, Some(normal_index - 1), None))
		} else {
			let vertex_index = parse_face_component_index(parts[0])?;
			let texture_coordinate_index = parse_face_component_index(parts[1])?;
			let normal_index = parse_face_component_index(parts[2])?;

			Ok((
				vertex_index - 1,
				Some(normal_index - 1),
				Some(texture_coordinate_index - 1),
			))
		}
	} else {
		Err(String::from(
			"Face vertices cannot have more than 3 components",
		))
	}
}

fn parse_face_component_index(part: &str) -> Result<usize, String>
{
	match part.parse::<usize>() {
		Ok(value) => Ok(value),
		Err(_) => {
			return Err(format!("Failed to parse index \"{}\"", part));
		},
	}
}
