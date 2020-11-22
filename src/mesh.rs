use na::{Matrix3, Matrix4, Vector4};
use std::f32;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use traits::Primitive;
use util::math;
use {BoundingBox, Hit, Ray};

pub struct Mesh {
    vertices: Vec<Vector4<f32>>,
    faces: Vec<Triangle>,
    bounding_box: BoundingBox,
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

            let mut line_iter = line.split_whitespace();

            if let Some(data_type) = line_iter.next() {
                match data_type {
                    "v" => {
                        let x = line_iter.next().unwrap().parse::<f32>().unwrap();
                        let y = line_iter.next().unwrap().parse::<f32>().unwrap();
                        let z = line_iter.next().unwrap().parse::<f32>().unwrap();

                        vertices.push(Vector4::new(x, y, z, 1.0));
                    }
                    "f" => {
                        let v1 = line_iter.next().unwrap().parse::<usize>().unwrap();
                        let v2 = line_iter.next().unwrap().parse::<usize>().unwrap();
                        let v3 = line_iter.next().unwrap().parse::<usize>().unwrap();

                        faces.push(Triangle {
                            v1: v1 - 1,
                            v2: v2 - 1,
                            v3: v3 - 1,
                        });
                    }
                    _ => {}
                };
            }
        }

        let mut max = Vector4::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);
        let mut min = Vector4::new(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);

        for vertex in &vertices {
            if min.x > vertex.x {
                min.x = vertex.x
            };
            if min.y > vertex.y {
                min.y = vertex.y
            };
            if min.z > vertex.z {
                min.z = vertex.z
            };

            if max.x < vertex.x {
                max.x = vertex.x
            };
            if max.y < vertex.y {
                max.y = vertex.y
            };
            if max.z < vertex.z {
                max.z = vertex.z
            };
        }

        Mesh {
            vertices: vertices,
            faces: faces,
            bounding_box: BoundingBox::new(&min, &max),
        }
    }
}

impl Primitive for Mesh {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit> {
        if !self.bounding_box.hit(ray, transform) {
            return None;
        }

        let point = transform * ray.point;
        let origin = transform * ray.origin;
        let op = origin - point;

        let mut hit_info: Option<(&Vector4<f32>, &Vector4<f32>, &Vector4<f32>, f32)> = None;

        for face in &self.faces {
            let p0 = &self.vertices[face.v1];
            let p1 = &self.vertices[face.v2];
            let p2 = &self.vertices[face.v3];

            let r = [origin.x - p0.x, origin.y - p0.y, origin.z - p0.z];
            let x = [p1.x - p0.x, p2.x - p0.x, op.x];
            let y = [p1.y - p0.y, p2.y - p0.y, op.y];
            let z = [p1.z - p0.z, p2.z - p0.z, op.z];

            let d =
                Matrix3::new(x[0], x[1], x[2], y[0], y[1], y[2], z[0], z[1], z[2]).determinant();

            let d1 =
                Matrix3::new(r[0], x[1], x[2], r[1], y[1], y[2], r[2], z[1], z[2]).determinant();

            let d2 =
                Matrix3::new(x[0], r[0], x[2], y[0], r[1], y[2], z[0], r[2], z[2]).determinant();

            let d3 =
                Matrix3::new(x[0], x[1], r[0], y[0], y[1], r[1], z[0], z[1], r[2]).determinant();

            let beta = d1 / d;
            let gamma = d2 / d;
            let nt = d3 / d;

            if beta >= 0.0 && gamma >= 0.0 && beta + gamma <= 1.0 && nt > math::EPSILON {
                if let Some((_, _, _, t)) = hit_info {
                    if nt < t {
                        hit_info = Some((p0, p1, p2, nt))
                    }
                } else {
                    hit_info = Some((p0, p1, p2, nt))
                }
            }
        }

        if let Some((p0, p1, p2, t)) = hit_info {
            let mut n = math::cross_4d(&(p1 - p0), &(p2 - p0));
            if op.dot(&n) < 0.0 {
                n = -n;
            }

            n = transform.transpose() * n;
            n.w = 0.0;

            Some(Hit {
                normal: n,
                intersect: t,
                uv: (0.0, 0.0),
            })
        } else {
            None
        }
    }
}
