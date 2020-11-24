use na::{Matrix4, Vector3, Vector4};
use std::f32;
use traits::Primitive;
use util::math;
use {Hit, Ray};
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
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit> {
        let point = transform * ray.point;
        let origin = transform * ray.origin;

        let po = point - origin;
        let oc = origin - self.position;

        let a = po.dot(&po);
        let b = po.dot(&oc) * 2.0;
        let c = oc.dot(&oc) - (self.radius * self.radius);

        match math::quadratic_roots(a, b, c) {
            math::QuadRoots::Zero | math::QuadRoots::One(_) => None,
            math::QuadRoots::Two(root_one, root_two) => {
                if root_one < math::EPSILON && root_two < math::EPSILON {
                    None
                } else {
                    let t = if root_one <= root_two {
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

                    let mut n = (origin + (t * po)) - self.position;

                    // Invert normal if inside sphere
                    if n.dot(&(origin - point)) < 0.0 {
                        n = -n;
                    }

                    n = transform.transpose() * n;
                    n.w = 0.0;

                    Some(Hit {
                        normal: n,
                        intersect: t,
                        uv: (0.0, 0.0),
                    })
                }
            }
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
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit> {
        enum Faces {
            Front,
            Back,
            Top,
            Bottom,
            Left,
            Right
        }

        let point = transform * ray.point;
        let origin = transform * ray.origin;

        let ray_direction = point - origin;
        let inv_direction = Vector4::repeat(1.0).component_div(&ray_direction);

        let mut normal_axis_min = Vector4::new(1.0, 0.0, 0.0, 0.0);
        let mut normal_axis_max = Vector4::new(1.0, 0.0, 0.0, 0.0);

        let (mut t_min, mut t_max) = 
        if inv_direction.x >= 0.0 {
            let t_min = (self.position.x - origin.x) * inv_direction.x;
            let t_max = (self.position.x + self.size - origin.x) * inv_direction.x;

            (t_min, t_max)
        } else {
            let t_min = (self.position.x + self.size - origin.x) * inv_direction.x;
            let t_max = (self.position.x - origin.x) * inv_direction.x;

            (t_min, t_max)
        };

        let (ty_min, ty_max) = 
        if inv_direction.y >= 0.0 {
            let ty_min = (self.position.y - origin.y) * inv_direction.y;
            let ty_max = (self.position.y + self.size - origin.y) * inv_direction.y;

            (ty_min, ty_max)
        } else {
            let ty_min = (self.position.y + self.size - origin.y) * inv_direction.y;
            let ty_max = (self.position.y - origin.y) * inv_direction.y;

            (ty_min, ty_max)
        };

        if (t_min > ty_max) || (ty_min > t_max) {
            return None
        }

        if ty_min > t_min {
            normal_axis_min = Vector4::new(0.0, 1.0, 0.0, 0.0);
            t_min = ty_min;
        }

        if ty_max < t_max {
            normal_axis_max = Vector4::new(0.0, 1.0, 0.0, 0.0);
            t_max = ty_max;
        }

        let (tz_min, tz_max) = 
        if inv_direction.z >= 0.0 {
            let tz_min = (self.position.z - origin.z) * inv_direction.z;
            let tz_max = (self.position.z + self.size - origin.z) * inv_direction.z;

            (tz_min, tz_max)
        } else {
            let tz_min = (self.position.z + self.size - origin.z) * inv_direction.z;
            let tz_max = (self.position.z - origin.z) * inv_direction.z;

            (tz_min, tz_max)
        };

        if (t_min > tz_max) || (tz_min > t_max) {
            return None
        }

        if tz_min > t_min {
            normal_axis_min = Vector4::new(0.0, 0.0, 1.0, 0.0);
            t_min = tz_min;
        }

        if tz_max < t_max {
            normal_axis_max = Vector4::new(0.0, 0.0, 1.0, 0.0);
            t_max = tz_max;
        }

        let (contact_point, normal_axis) =
        if t_min > math::EPSILON {
            (t_min, normal_axis_min)
        } else if t_max > math::EPSILON {
            (t_max, normal_axis_max)
        } else {
            return None;
        };
        
        let normal = transform.transpose() * (-ray_direction).component_mul(&normal_axis).normalize();

        Some(Hit {
            intersect: contact_point,
            normal: normal, 
            uv: (0.0, 0.0)
        })
    }

        // let point = transform * ray.point;
        // let origin = transform * ray.origin;
        // let position = &self.position;
        // let size = self.size;

        // let mut hit_info: Option<(Vector4<f32>, f32, f32, f32)> = None;

        // for i in 0..6 {
        //     let (p0, p1, p2) = match i {
        //         0 => {
        //             let p0 = Vector4::<f32>::new(position.x, position.y, position.z, 1.0);
        //             let p1 = Vector4::<f32>::new(position.x + size, position.y, position.z, 1.0);
        //             let p2 = Vector4::<f32>::new(position.x, position.y + size, position.z, 1.0);
        //             (p0, p1, p2)
        //         }
        //         1 => {
        //             let p0 = Vector4::<f32>::new(position.x, position.y, position.z + size, 1.0);
        //             let p1 =
        //                 Vector4::<f32>::new(position.x, position.y + size, position.z + size, 1.0);
        //             let p2 =
        //                 Vector4::<f32>::new(position.x + size, position.y, position.z + size, 1.0);
        //             (p0, p1, p2)
        //         }
        //         2 => {
        //             let p0 = Vector4::<f32>::new(position.x, position.y, position.z, 1.0);
        //             let p1 = Vector4::<f32>::new(position.x, position.y + size, position.z, 1.0);
        //             let p2 = Vector4::<f32>::new(position.x, position.y, position.z + size, 1.0);
        //             (p0, p1, p2)
        //         }
        //         3 => {
        //             let p0 = Vector4::<f32>::new(position.x + size, position.y, position.z, 1.0);
        //             let p1 =
        //                 Vector4::<f32>::new(position.x + size, position.y, position.z + size, 1.0);
        //             let p2 =
        //                 Vector4::<f32>::new(position.x + size, position.y + size, position.z, 1.0);
        //             (p0, p1, p2)
        //         }
        //         4 => {
        //             let p0 = Vector4::<f32>::new(position.x, position.y, position.z, 1.0);
        //             let p1 = Vector4::<f32>::new(position.x, position.y, position.z + size, 1.0);
        //             let p2 = Vector4::<f32>::new(position.x + size, position.y, position.z, 1.0);
        //             (p0, p1, p2)
        //         }
        //         5 => {
        //             let p0 = Vector4::<f32>::new(position.x, position.y + size, position.z, 1.0);
        //             let p1 =
        //                 Vector4::<f32>::new(position.x + size, position.y + size, position.z, 1.0);
        //             let p2 =
        //                 Vector4::<f32>::new(position.x, position.y + size, position.z + size, 1.0);
        //             (p0, p1, p2)
        //         }
        //         _ => panic!("This should never happen"),
        //     };

        //     let nn = math::cross_4d(&(p1 - p0), &(p2 - p0));

        //     let la = (origin - p0).dot(&nn);
        //     let lb = (point - p0).dot(&nn);
        //     let nt = la / (la - lb);

        //     let t = if let Some((_, t, _, _)) = hit_info {
        //         t
        //     } else {
        //         f32::INFINITY
        //     };

        //     if nt < t && nt > math::EPSILON {
        //         let pt = origin + (nt * (point - origin));

        //         match i {
        //             0 | 1 => {
        //                 let diff_x = pt.x - p0.x;
        //                 let diff_y = pt.y - p0.y;
        //                 if diff_x <= size && diff_x >= 0.0 && diff_y <= size && diff_y >= 0.0 {
        //                     let mut u = diff_x / size;
        //                     let v = diff_y / size;

        //                     if i == 1 {
        //                         u = 1.0 - u;
        //                     }

        //                     hit_info = Some((nn, nt, u, v));
        //                 }
        //             }
        //             2 | 3 => {
        //                 let diff_z = pt.z - p0.z;
        //                 let diff_y = pt.y - p0.y;
        //                 if diff_z <= size && diff_z >= 0.0 && diff_y <= size && diff_y >= 0.0 {
        //                     let mut u = diff_z / size;
        //                     let v = diff_y / size;

        //                     if i == 2 {
        //                         u = 1.0 - u;
        //                     }

        //                     hit_info = Some((nn, nt, u, v));
        //                 }
        //             }
        //             4 | 5 => {
        //                 let diff_x = pt.x - p0.x;
        //                 let diff_z = pt.z - p0.z;
        //                 if diff_x <= size && diff_x >= 0.0 && diff_z <= size && diff_z >= 0.0 {
        //                     let u = diff_x / size;
        //                     let v = diff_z / size;

        //                     hit_info = Some((nn, nt, u, v));
        //                 }
        //             }
        //             _ => panic!("This should never happen"),
        //         };
        //     }
        // }

        // if let Some((mut normal, intersect, u, v)) = hit_info {
        //     // Invert normal if inside box
        //     if (origin - point).dot(&normal) < 0.0 {
        //         normal = -normal;
        //     }

        //     normal = transform.transpose() * normal;
        //     normal.w = 0.0;

        //     let normalx = normal.x;
        //     let normaly = normal.y;
        //     let normalz = normal.z;

        //     Some(Hit {
        //         normal: normal,
        //         intersect: intersect,
        //         uv: (u, v),
        //     })
        // } else {
        //     None
        // }
    // }
}

pub struct Sphere {}

impl Sphere {
    pub fn new() -> Self {
        Sphere {}
    }
}

impl Primitive for Sphere {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit> {
        let point = transform * ray.point;
        let origin = transform * ray.origin;

        let po = point - origin;
        let oc = origin - Vector4::new(0.0, 0.0, 0.0, 1.0);

        let a = po.dot(&po);
        let b = po.dot(&oc) * 2.0;
        let c = oc.dot(&oc) - 1.0;

        match math::quadratic_roots(a, b, c) {
            math::QuadRoots::Zero | math::QuadRoots::One(_) => None,
            math::QuadRoots::Two(root_one, root_two) => {
                if root_one < math::EPSILON && root_two < math::EPSILON {
                    None
                } else {
                    let t = if root_one <= root_two {
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

                    let mut n = origin + (t * po);

                    // Invert normal if inside sphere
                    if n.dot(&(origin - point)) < 0.0 {
                        n = -n;
                    }

                    n = transform.transpose() * n;
                    n.w = 0.0;

                    Some(Hit {
                        normal: n,
                        intersect: t,
                        uv: (0.0, 0.0),
                    })
                }
            }
        }
    }
}

pub struct Cube {}

impl Cube {
    pub fn new() -> Self {
        Cube {}
    }
}

impl Primitive for Cube {
    fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> Option<Hit> {
        let point = transform * ray.point;
        let origin = transform * ray.origin;

        let mut hit_info: Option<(Vector4<f32>, f32, f32, f32)> = None;

        for i in 0..6 {
            let (p0, p1, p2) = match i {
                0 => {
                    let p0 = Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0);
                    let p1 = Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0);
                    let p2 = Vector4::<f32>::new(0.0, 1.0, 0.0, 1.0);
                    (p0, p1, p2)
                }
                1 => {
                    let p0 = Vector4::<f32>::new(0.0, 0.0, 1.0, 1.0);
                    let p1 = Vector4::<f32>::new(0.0, 1.0, 1.0, 1.0);
                    let p2 = Vector4::<f32>::new(1.0, 0.0, 1.0, 1.0);
                    (p0, p1, p2)
                }
                2 => {
                    let p0 = Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0);
                    let p1 = Vector4::<f32>::new(0.0, 1.0, 0.0, 1.0);
                    let p2 = Vector4::<f32>::new(0.0, 0.0, 1.0, 1.0);
                    (p0, p1, p2)
                }
                3 => {
                    let p0 = Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0);
                    let p1 = Vector4::<f32>::new(1.0, 0.0, 1.0, 1.0);
                    let p2 = Vector4::<f32>::new(1.0, 1.0, 0.0, 1.0);
                    (p0, p1, p2)
                }
                4 => {
                    let p0 = Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0);
                    let p1 = Vector4::<f32>::new(0.0, 0.0, 1.0, 1.0);
                    let p2 = Vector4::<f32>::new(1.0, 0.0, 0.0, 1.0);
                    (p0, p1, p2)
                }
                5 => {
                    let p0 = Vector4::<f32>::new(0.0, 1.0, 0.0, 1.0);
                    let p1 = Vector4::<f32>::new(1.0, 1.0, 0.0, 1.0);
                    let p2 = Vector4::<f32>::new(0.0, 1.0, 1.0, 1.0);
                    (p0, p1, p2)
                }
                _ => panic!("This should never happen"),
            };

            let nn = math::cross_4d(&(p1 - p0), &(p2 - p0));

            let la = (origin - p0).dot(&nn);
            let lb = (point - p0).dot(&nn);
            let nt = la / (la - lb);

            let t = if let Some((_, t, _, _)) = hit_info {
                t
            } else {
                f32::INFINITY
            };

            if nt < t && nt > math::EPSILON {
                let pt = origin + (nt * (point - origin));

                match i {
                    0 | 1 => {
                        let diff_x = pt.x - p0.x;
                        let diff_y = pt.y - p0.y;
                        if diff_x <= 1.0 && diff_x >= 0.0 && diff_y <= 1.0 && diff_y >= 0.0 {
                            let mut u = diff_x;
                            let v = diff_y;

                            if i == 1 {
                                u = 1.0 - u;
                            }

                            hit_info = Some((nn, nt, u, v));
                        }
                    }
                    2 | 3 => {
                        let diff_z = pt.z - p0.z;
                        let diff_y = pt.y - p0.y;
                        if diff_z <= 1.0 && diff_z >= 0.0 && diff_y <= 1.0 && diff_y >= 0.0 {
                            let mut u = diff_z;
                            let v = diff_y;

                            if i == 2 {
                                u = 1.0 - u;
                            }

                            hit_info = Some((nn, nt, u, v));
                        }
                    }
                    4 | 5 => {
                        let diff_x = pt.x - p0.x;
                        let diff_z = pt.z - p0.z;
                        if diff_x <= 1.0 && diff_x >= 0.0 && diff_z <= 1.0 && diff_z >= 0.0 {
                            let u = diff_x;
                            let v = diff_z;

                            hit_info = Some((nn, nt, u, v));
                        }
                    }
                    _ => panic!("This should never happen"),
                };
            }
        }

        if let Some((mut normal, intersect, u, v)) = hit_info {
            // Invert normal if inside box
            if (origin - point).dot(&normal) < 0.0 {
                normal = -normal;
            }

            normal = transform.transpose() * normal;
            normal.w = 0.0;

            Some(Hit {
                normal: normal,
                intersect: intersect,
                uv: (u, v),
            })
        } else {
            None
        }
    }
}
