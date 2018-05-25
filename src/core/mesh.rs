use core::traits::Primitive;
use core::Ray;
use core::math;
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;
use std::f32;
use na::{Matrix4, Matrix3, Vector4, Vector3};

pub struct Mesh {
    vertices: Vec<Vector4<f32>>,
    faces: Vec<Triangle>,
}

struct Triangle {
    pub v1: usize,
    pub v2: usize,
    pub v3: usize,
}

impl Mesh {
    pub fn new(file_name: &String) -> Self {
        let obj_file = File::open(file_name).unwrap();
        let reader = BufReader::new(obj_file);

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for line in reader.lines() {
            let line = line.unwrap();
            let mut line_array = line.split(' ');

            match line_array.next().unwrap() {
                "v" => {
                    let x = line_array.next().unwrap().parse::<f32>().unwrap();
                    let y = line_array.next().unwrap().parse::<f32>().unwrap();
                    let z = line_array.next().unwrap().parse::<f32>().unwrap();

                    vertices.push(Vector4::new(x, y, z, 1.0));
                },
                "f" => {
                    let v1 = line_array.next().unwrap().parse::<usize>().unwrap();
                    let v2 = line_array.next().unwrap().parse::<usize>().unwrap();
                    let v3 = line_array.next().unwrap().parse::<usize>().unwrap();

                    faces.push(Triangle { v1: v1 - 1, v2: v2 - 1, v3: v3 - 1 });
                },
                _ => {},
            };
        }

        Mesh {
            vertices: vertices,
            faces: faces,
        }
    }
}

impl Primitive for Mesh {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>, intersect: &mut f32, normal: &mut Vector4<f32>, u: &mut f32, v: &mut f32) -> bool {
        let point = transform * ray.point;
        let origin = transform * ray.origin;

        let mut hit = false;
        let mut t = f32::INFINITY;

        for face in &self.faces {
            let p0 = self.vertices[face.v1];
            let p1 = self.vertices[face.v2];
            let p2 = self.vertices[face.v3];

            let r = Vector3::new(origin.x - p0.x, origin.y - p0.y, origin.z - p0.z);
            let x = Vector3::new(p1.x - p0.x, p2.x - p0.x, origin.x - point.x);
            let y = Vector3::new(p1.y - p0.y, p2.y - p0.y, origin.y - point.y);
            let z = Vector3::new(p1.z - p0.z, p2.z - p0.z, origin.z - point.z);

            let d  = Matrix3::from_columns(&vec!(Vector3::new(x.x, y.x, z.x), Vector3::new(x.y, y.y, z.y), Vector3::new(x.z, y.z, z.z))).determinant();
            let d1 = Matrix3::from_columns(&vec!(r, Vector3::new(x.y, y.y, z.y), Vector3::new(x.z, y.z, z.z))).determinant();
            let d2 = Matrix3::from_columns(&vec!(Vector3::new(x.x, y.x, z.x), r, Vector3::new(x.z, y.z, z.z))).determinant();
            let d3 = Matrix3::from_columns(&vec!(Vector3::new(x.x, y.x, z.x), Vector3::new(x.y, y.y, z.y), r)).determinant();

            let beta = d1 / d;
            let gamma = d2 / d;
            let nt = d3 / d;

            if beta >= 0.0 && gamma >= 0.0 && beta + gamma <= 1.0 && nt < t && nt > math::EPSILON {
                let mut n = math::cross_4d(&(p1 - p0), &(p2 - p0));

                if (origin - point).dot(&n) < 0.0 {
                    n = -n;
                }

                hit = true;
                t = nt;
                *intersect = t;
                *normal = transform.transpose() * n;
                normal.w = 0.0;
                *u = 0.0;
                *v = 0.0;
            }
        }

        hit
    }
}