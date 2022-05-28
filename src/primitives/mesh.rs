use std::f32;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::error::Error;

use na::Matrix4;
use na::Vector4;
use primitives::Primitive;
use util::math;
use Hit;
use Ray;

#[derive(fmt::Debug)]
pub struct ObjParseError
{
	message: String
}

impl ObjParseError
{
	pub fn new(message: String) -> Self
	{
		ObjParseError {
			message: message
		}
	}
}

impl fmt::Display for ObjParseError
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
	{
		write!(f, "ObjParseError: {}", self.message)
	}
}

impl Error for ObjParseError {}

#[derive(fmt::Debug)]
pub struct Mesh
{
	vertices: Vec<Vector4<f32>>,
	normals: Vec<Vector4<f32>>,
	texture_coordinates: Vec<Vector4<f32>>,
	faces: Vec<Triangle>,
}

#[derive(fmt::Debug)]
struct Triangle
{
	pub vertices: (usize, usize, usize),
	pub normals: Option<(usize, usize, usize)>,
	pub texture_coordinates: Option<(usize, usize, usize)>
}

impl Mesh
{
	pub fn from_file(file_name: &String) -> Result<Self, ObjParseError>
	{
		let mut vertices = Vec::new();
		let mut normals = Vec::new();
		let mut texture_coordinates = Vec::new();
		let mut faces = Vec::new();

		let obj_file = match File::open(file_name) {
			Ok(file) => file,
			Err(error) => return Err(ObjParseError::new(format!("Couldn't open mesh file {}: {}", file_name, error)))
		};

		let reader = BufReader::new(obj_file);

		for line in reader.lines() {
			let line = match line {
				Ok(line) => line,
				Err(error) => return Err(ObjParseError::new(format!("File {} closed unexpectedly: {}", file_name, error)))
			};

			let line = line.trim();

			match line.chars().nth(0) {
				Some(char) => if char == '#' { continue; },
				None => continue,
			};

			let line_parts: Vec<&str> = line.split_whitespace().collect();

			if line_parts.len() >= 1 {
				match line_parts[0] {
					"v" => {
						let vertex = Mesh::parse_vertex_data(&line_parts)?;
						vertices.push(vertex);
					}
					"vn" => {
						let normal = Mesh::parse_vertex_data(&line_parts)?;
						normals.push(normal);
					}
					"vt" => {
						let texture_coordinate = Mesh::parse_vertex_data(&line_parts)?;
						texture_coordinates.push(texture_coordinate);
					}
					"f" => {
						let face = Mesh::parse_face_data(&line_parts)?;
						faces.push(face);
					},
					"o" | "g" | "s" => {
						// Object names (o), groups (g), smoothing groups (s) are ignored
					},
					_ => {
						return Err(ObjParseError::new(format!("\"{}\" tag not supported", line_parts[0])))
					}
				};
			}
		}

		println!("Num Vertices: {}", vertices.len());
		println!("Num Normals: {}", normals.len());

		Ok(Mesh {
			vertices: vertices,
			normals: normals,
			texture_coordinates: texture_coordinates,
			faces: faces,
		})
	}

	fn parse_vertex_data(parts: &Vec<&str>) -> Result<Vector4<f32>, ObjParseError>
	{
		if parts.len() < 4 {
			return Err(ObjParseError::new(String::from("Vertices with fewer than 3 coordinates are not supported")));
		} else if parts.len() > 4 {
			return Err(ObjParseError::new(String::from("Vertices with more than 3 coordinates are not supported")));
		}

		let x = match parts[1].parse::<f32>() {
			Ok(value) => value,
			Err(error) => {
				return Err(ObjParseError::new(format!("Failed to parse vertex data as f32: {}", error)));
			}
		};

		let y = match parts[2].parse::<f32>() {
			Ok(value) => value,
			Err(error) => {
				return Err(ObjParseError::new(format!("Failed to parse vertex data as f32: {}", error)));
			}
		};

		let z = match parts[3].parse::<f32>() {
			Ok(value) => value,
			Err(error) => {
				return Err(ObjParseError::new(format!("Failed to parse vertex data as f32: {}", error)));
			}
		};

		Ok(Vector4::new(x, y, z, 1.0))
	}

	fn parse_face_data(parts: &Vec<&str>) -> Result<Triangle, ObjParseError>
	{
		if parts.len() < 4 {
			return Err(ObjParseError::new(String::from("Faces with fewer than 3 vertices are not supported")));
		} else if parts.len() > 4 {
			return Err(ObjParseError::new(String::from("Faces with more than 3 vertices are not supported")));
		}

		let v1_data = Mesh::parse_face_component(parts[1])?;
		let v2_data = Mesh::parse_face_component(parts[2])?;
		let v3_data = Mesh::parse_face_component(parts[3])?;

		let vertices = (v1_data.0, v2_data.0, v3_data.0);

		let normals = if v1_data.1.is_some() && v2_data.1.is_some() && v3_data.1.is_some() {
			Some((v1_data.1.unwrap(), v2_data.1.unwrap(), v3_data.1.unwrap()))
		} else if v1_data.1.is_none() && v2_data.1.is_none() && v3_data.1.is_none() {
			None
		} else {
			return Err(ObjParseError::new(String::from("Some vertices missing normal data")))
		};

		let texture_coordinates = if v1_data.2.is_some() && v2_data.2.is_some() && v3_data.2.is_some() {
			Some((v1_data.2.unwrap(), v2_data.2.unwrap(), v3_data.2.unwrap()))
		} else if v1_data.2.is_none() && v2_data.2.is_none() && v3_data.2.is_none() {
			None
		} else {
			return Err(ObjParseError::new(String::from("Some vertices missing texture coordinate data")))
		};

		Ok(Triangle {
			vertices: vertices,
			normals: normals,
			texture_coordinates: texture_coordinates
		})
	}

	fn parse_face_component(component: &str) -> Result<(usize, Option<usize>, Option<usize>), ObjParseError>
	{
		let parts: Vec<&str> = component.split('/').collect();

		if parts.len() == 1 {
			let vertex_index = match parts[0].parse::<usize>() {
				Ok(value) => value,
				Err(_) => {
					return Err(ObjParseError::new(String::from("Failed to parse vertex index as usize")));
				}
			};

			Ok((vertex_index - 1, None, None))
		} else if parts.len() == 2 {
			let vertex_index = match parts[0].parse::<usize>() {
				Ok(value) => value,
				Err(_) => {
					return Err(ObjParseError::new(String::from("Failed to parse vertex index as usize")));
				}
			};

			let texture_coordinate_index = match parts[1].parse::<usize>() {
				Ok(value) => value,
				Err(_) => {
					return Err(ObjParseError::new(String::from("Failed to parse texture coordinate index as usize")));
				}
			};

			Ok((vertex_index - 1, None, Some(texture_coordinate_index - 1)))
		} else if parts.len() == 3 {
			if parts[1].is_empty() {
				let vertex_index = match parts[0].parse::<usize>() {
					Ok(value) => value,
					Err(_) => {
						return Err(ObjParseError::new(String::from("Failed to parse vertex index as usize")));
					}
				};
	
				let normal_index = match parts[2].parse::<usize>() {
					Ok(value) => value,
					Err(_) => {
						return Err(ObjParseError::new(String::from("Failed to parse normal index as usize")));
					}
				};
	
				Ok((vertex_index - 1, Some(normal_index - 1), None))
			} else {
				let vertex_index = match parts[0].parse::<usize>() {
					Ok(value) => value,
					Err(_) => {
						return Err(ObjParseError::new(String::from("Failed to parse vertex index as usize")));
					}
				};

				let texture_coordinate_index = match parts[1].parse::<usize>() {
					Ok(value) => value,
					Err(_) => {
						return Err(ObjParseError::new(String::from("Failed to parse texture coordinate index as usize")));
					}
				};
	
				let normal_index = match parts[2].parse::<usize>() {
					Ok(value) => value,
					Err(_) => {
						return Err(ObjParseError::new(String::from("Failed to parse normal index as usize")));
					}
				};
	
				Ok((vertex_index - 1, Some(normal_index - 1), Some(texture_coordinate_index - 1)))
			}
		} else {
			Err(ObjParseError::new(String::from("")))
		}
	}
}

impl Primitive for Mesh
{
	fn hit(&self, ray: &Ray, transform: Matrix4<f32>) -> Option<Hit>
	{
		let point = transform * ray.point();
		let origin = transform * ray.origin();
		let vector = point - origin;

		let mut intersect = f32::INFINITY;
		let mut normal = Vector4::new(0.0, 0.0, 0.0, 0.0);

		for face in &self.faces {
			// Moller-Trombore intersection algorithm

			let v1 = &self.vertices[face.vertices.0];
			let v2 = &self.vertices[face.vertices.1];
			let v3 = &self.vertices[face.vertices.2];

			let edge1 = v2 - v1;
			let edge2 = v3 - v1;

			let h = math::cross_4d(vector, edge2);
			let a = edge1.dot(&h);

			if math::near_zero(a) {
				continue;
			}

			let f = 1.0 / a;
			let s = origin - v1;
			let u = f * s.dot(&h);

			if u < 0.0 || u > 1.0 {
				continue;
			}

			let q = math::cross_4d(s, edge1);
			let v = f * vector.dot(&q);

			if v < 0.0 || u + v > 1.0 {
				continue;
			}

			let t = f * edge2.dot(&q);

			if !math::far_from_zero_pos(t) {
				continue;
			}

			if t < intersect {
				intersect = t;

				if let Some(normals) = face.normals {
					let n1 = &self.normals[normals.0];
					let n2 = &self.normals[normals.1];
					let n3 = &self.normals[normals.2];

					normal = ((n1 * (1.0 - u - v)) + (n2 * u) + (n3 * v)).normalize();
				} else {
					normal = math::cross_4d(v2 - v1, v3 - v1).normalize();
				}
			}
		}

		if intersect < f32::INFINITY {
			if vector.dot(&normal) > 0.0 {
				normal = -normal;
			}

			normal = math::transform_normals(normal, transform);

			Some(Hit {
				normal: normal,
				intersect: intersect,
				uv: (0.0, 0.0),
			})
		} else {
			None
		}
	}

	fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>)
	{
		let mut max = Vector4::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);
		let mut min = Vector4::new(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);

		for vertex in &self.vertices {
			min.x = f32::min(min.x, vertex.x);
			min.y = f32::min(min.y, vertex.y);
			min.z = f32::min(min.z, vertex.z);

			max.x = f32::max(max.x, vertex.x);
			max.y = f32::max(max.y, vertex.y);
			max.z = f32::max(max.z, vertex.z);
		}

		(min, max)
	}
}
