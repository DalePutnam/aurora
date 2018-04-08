use na::{Vector4, Vector3, Matrix4, U3, U1, dot};
use core::ray::Ray;
use core::math;
use std::f32;

pub trait Primitive: Send + Sync {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>, intersect: &mut f32, normal: &mut Vector4<f32>, u: &mut f32, v: &mut f32) -> bool;
}

pub struct NonhierSphere {
    position: Vector4<f32>,
    radius: f32,
}

impl NonhierSphere {
    pub fn new(position: Vector3<f32>, radius: f32) -> Self {
        NonhierSphere {
            position: Vector4::new(position.x, position.y, position.z, 1.0),
            radius: radius,
        }
    }
}

impl Primitive for NonhierSphere {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>, intersect: &mut f32, normal: &mut Vector4<f32>, u: &mut f32, v: &mut f32) -> bool {
        let point = transform * ray.point;
        let origin = transform * ray.origin;

        let po = point - origin;
        let oc = origin - self.position;

        let a = dot(&po, &po);
        let b = dot(&po, &oc) * 2.0;
        let c = dot(&oc, &oc) - (self.radius * self.radius);

        match math::quadratic_roots(a, b, c) {
            math::QuadRoots::Zero | math::QuadRoots::One(_) => false,
            math::QuadRoots::Two(root_one, root_two) => {
                if root_one < math::EPSILON && root_two < math::EPSILON {
                    false
                } else {
                    *intersect = if root_one <= root_two { 
                        if root_one > math::EPSILON {
                            root_one
                        } else {
                            root_two
                        }
                    } else {
                        if root_two > math::EPSILON {
                            root_two
                        } else {
                            root_one
                        }
                    };

                    let n = (origin + (*intersect * po)) - self.position;

                    // TODO: Calculate UVs
                    *u = 0.0;
                    *v = 0.0;

                    *normal = transform.transpose() * n;

                    true
                }
            },
        }
    }
}

pub struct NonhierBox {
    position: Vector4<f32>,
    size: f32,
}

impl NonhierBox {
    pub fn new(position: Vector3<f32>, size: f32) -> Self {
        NonhierBox {
            position: Vector4::new(position.x, position.y, position.z, 1.0),
            size: size,
        }
    }
}

impl Primitive for NonhierBox {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>, intersect: &mut f32, normal: &mut Vector4<f32>, u: &mut f32, v: &mut f32) -> bool {
        let point = transform * ray.point;
        let origin = transform * ray.origin;
        let position = &self.position;
        let size = self.size;

        let hit = false;
        let t = f32::MAX;
        let nu = 0.0;
        let nv = 0.0;

        for i in 0..6 {
            let (p0, p1, p2) = match i {
                0 => {
                    let p0 = Vector4::<f32>::new(position.x, position.y, position.z, 1.0);
                    let p1 = Vector4::<f32>::new(position.x + size, position.y, position.z, 1.0);
                    let p2 = Vector4::<f32>::new(position.x, position.y + size, position.z, 1.0);
                    (p0, p1, p2)
                },
                1 => {
                    let p0 = Vector4::<f32>::new(position.x, position.y, position.z + size, 1.0);
                    let p1 = Vector4::<f32>::new(position.x, position.y + size, position.z + size, 1.0);
                    let p2 = Vector4::<f32>::new(position.x + size, position.y, position.z + size, 1.0);
                    (p0, p1, p2)
                },
                2 => {
                    let p0 = Vector4::<f32>::new(position.x, position.y, position.z, 1.0);
                    let p1 = Vector4::<f32>::new(position.x, position.y + size, position.z, 1.0);
                    let p2 = Vector4::<f32>::new(position.x, position.y, position.z + size, 1.0);
                    (p0, p1, p2)
                },
                3 => {
                    let p0 = Vector4::<f32>::new(position.x + size, position.y, position.z, 1.0);
                    let p1 = Vector4::<f32>::new(position.x + size, position.y, position.z + size, 1.0);
                    let p2 = Vector4::<f32>::new(position.x + size, position.y + size, position.z, 1.0);
                    (p0, p1, p2)
                },
                4 => {
                    let p0 = Vector4::<f32>::new(position.x, position.y, position.z, 1.0);
                    let p1 = Vector4::<f32>::new(position.x, position.y, position.z + size, 1.0);
                    let p2 = Vector4::<f32>::new(position.x + size, position.y, position.z, 1.0);
                    (p0, p1, p2)
                },
                5 => {
                    let p0 = Vector4::<f32>::new(position.x, position.y + size, position.z, 1.0);
                    let p1 = Vector4::<f32>::new(position.x + size, position.y + size, position.z, 1.0);
                    let p2 = Vector4::<f32>::new(position.x, position.y + size, position.z + size, 1.0);
                    (p0, p1, p2)
                },
                _ => panic!("This should never happen"),
            };

            //let mut nn = (p1 - p0).cross(&(p2 - p0));
            let mut nn = math::cross_4d(&(p1 - p0), &(p2 - p0));

            //let la = (origin.fixed_slice::<U3, U1>(0, 0) - p0).dot(&nn);
            //let lb = (point.fixed_slice::<U3, U1>(0, 0) - p0).dot(&nn);
            let la = (origin - p0).dot(&nn);
            let lb = (point - p0).dot(&nn);
            let nt = la / (la - lb);

            // Invert normal if inside box
            //if (origin - point).fixed_slice::<U3, U1>(0, 0).dot(&nn) < 0.0 {
            if (origin - point).dot(&nn) < 0.0 {
                nn = -nn;
            }

            if nt < t && nt > math::EPSILON {

            }
        }

        false
    }
}