use std::f32;
use std::fmt;
use std::error;
use std::path::Path;
use na::Matrix4;
use na::Vector4;
use primitives::Primitive;
use util::math;
use util::format::obj;
use Hit;
use Ray;

#[derive(fmt::Debug)]
pub enum Error {
	FileTypeError(String),
	FileReadError(Box<dyn error::Error + std::marker::Send + std::marker::Sync>),
}

impl Error {
	fn read_error<E: error::Error + std::marker::Send + std::marker::Sync + 'static>(err: E) -> Self {
		Self::FileReadError(Box::new(err))
	}

	fn type_error(msg: String) -> Self {
		Self::FileTypeError(msg)
	}
}

impl fmt::Display for Error
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>
	{
		match self {
			Self::FileReadError(err) => return err.fmt(f),
			Self::FileTypeError(msg) => return write!(f, "{}", msg),
		}
	}
}

impl error::Error for Error {}

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
	pub fn from_file(file_name: &String) -> Result<Self, Error>
	{
		let file_path = Path::new(file_name);

		if let Some(file_extension) = file_path.extension() {
			let file_extension = file_extension.to_string_lossy();

			match file_extension.as_ref() {
				obj::FILE_EXTENSION => {
					let obj_mesh = obj::read_file(file_name).map_err(Error::read_error)?;

					let faces = obj_mesh.f.iter().map(|face| {
						Triangle {
							vertices: face.v,
							normals: face.vn,
							texture_coordinates: face.vt
						}
					}).collect();
			
					return Ok(Mesh {
						vertices: obj_mesh.v,
						normals: obj_mesh.vn,
						texture_coordinates: obj_mesh.vt,
						faces: faces,
					});
				},
				_ => {
					// No-op, error returned later
				}
			}
		}

		Err(Error::type_error(format!("Failed to read {}: unknown mesh file type", file_name)))
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
