use na::{Matrix4, Vector4};
use std::f32;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use traits::Primitive;
use util::math;
use Hit;
use Ray;

pub struct Mesh {
    vertices: Vec<Vector4<f32>>,
    faces: Vec<Triangle>
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

        Mesh {
            vertices: vertices,
            faces: faces
        }
    }
}

impl Primitive for Mesh {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit> {
        let point = transform * ray.point;
        let origin = transform * ray.origin;
        let vector = point - origin;
        
        let mut intersect = f32::INFINITY;
        let mut normal = Vector4::new(0.0, 0.0, 0.0, 0.0);

        for face in &self.faces {
            // Moller-Trombore intersection algorithm

            let v1 = &self.vertices[face.v1];
            let v2 = &self.vertices[face.v2];
            let v3 = &self.vertices[face.v3];

            let edge1 = v2 - v1;
            let edge2 = v3 - v1;
            
            let h = math::cross_4d(&vector, &edge2);
            let a = edge1.dot(&h);

            if f32::abs(a) < math::EPSILON {
                continue;
            }

            let f = 1.0 / a;
            let s = origin - v1;
            let u = f * s.dot(&h);

            if u < 0.0 || u > 1.0 {
                continue;
            }

            let q = math::cross_4d(&s, &edge1);
            let v = f * vector.dot(&q);

            if v < 0.0 || u + v > 1.0 {
                continue;
            }

            let t = f * edge2.dot(&q);

            if t < math::EPSILON {
                continue;
            }

            if t < intersect {
                intersect = t;
                normal = math::cross_4d(&(v2 - v1), &(v3 - v1)).normalize();
            }
        }

        if intersect < f32::INFINITY {
            if vector.dot(&normal) > 0.0 {
                normal = -normal;
            }

            normal = math::local_to_world_normals(&normal, &transform);

            Some(Hit {
                normal: normal,
                intersect: intersect,
                uv: (0.0, 0.0)
            })
        }
        else
        {
            None
        }
    }

    fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>) {
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
