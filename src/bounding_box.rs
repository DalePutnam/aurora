use na::{Matrix4, Vector4};
use std::f32;
use util::math;
use Ray;

pub struct BoundingBox {
    lower_point: Vector4<f32>,
    upper_point: Vector4<f32>,
}

impl BoundingBox {
    pub fn new(lower_point: &Vector4<f32>, upper_point: &Vector4<f32>) -> Self {
        BoundingBox {
            lower_point: *lower_point,
            upper_point: *upper_point,
        }
    }

    pub fn hit(&self, ray: &Ray, transform: &Matrix4<f32>) -> bool {
        let point = transform * ray.point;
        let origin = transform * ray.origin;

        for i in 0..6 {
            let (p, n) = match i {
                0 => {
                    // Front face
                    let p = self.lower_point;
                    let n = Vector4::new(0.0, 0.0, -1.0, 0.0);
                    (p, n)
                }
                1 => {
                    // Back face
                    let p = Vector4::new(
                        self.lower_point.x,
                        self.lower_point.y,
                        self.upper_point.y,
                        1.0,
                    );
                    let n = Vector4::new(0.0, 0.0, 1.0, 0.0);
                    (p, n)
                }
                2 => {
                    // Left face
                    let p = self.lower_point;
                    let n = Vector4::new(-1.0, 0.0, 0.0, 0.0);
                    (p, n)
                }
                3 => {
                    // Right face
                    let p = Vector4::new(
                        self.upper_point.x,
                        self.lower_point.y,
                        self.lower_point.z,
                        1.0,
                    );
                    let n = Vector4::new(1.0, 0.0, 0.0, 0.0);
                    (p, n)
                }
                4 => {
                    // Bottom face
                    let p = self.lower_point;
                    let n = Vector4::new(0.0, -1.0, 0.0, 0.0);
                    (p, n)
                }
                5 => {
                    // Top face
                    let p = Vector4::new(
                        self.lower_point.x,
                        self.upper_point.y,
                        self.lower_point.z,
                        1.0,
                    );
                    let n = Vector4::new(0.0, 1.0, 0.0, 0.0);
                    (p, n)
                }
                _ => panic!("Unreachable!"),
            };

            let la = (origin - p).dot(&n);
            let lb = (point - p).dot(&n);
            let intersect = la / (la - lb);

            if intersect > math::EPSILON {
                let pt = origin + (intersect * (point - origin));

                match i {
                    0 | 1 => {
                        let size_x = self.upper_point.x - self.lower_point.x;
                        let size_y = self.upper_point.y - self.lower_point.y;

                        let diff_x = pt.x - p.x;
                        let diff_y = pt.y - p.y;

                        if diff_x <= size_x && diff_x >= 0.0 && diff_y <= size_y && diff_y >= 0.0 {
                            return true;
                        }
                    }
                    2 | 3 => {
                        let size_z = self.upper_point.z - self.lower_point.z;
                        let size_y = self.upper_point.y - self.lower_point.y;

                        let diff_z = pt.z - p.z;
                        let diff_y = pt.y - p.y;

                        if diff_z <= size_z && diff_z >= 0.0 && diff_y <= size_y && diff_y >= 0.0 {
                            return true;
                        }
                    }
                    4 | 5 => {
                        let size_x = self.upper_point.x - self.lower_point.x;
                        let size_z = self.upper_point.z - self.lower_point.z;

                        let diff_x = pt.x - p.x;
                        let diff_z = pt.z - p.z;

                        if diff_x <= size_x && diff_x >= 0.0 && diff_z <= size_z && diff_z >= 0.0 {
                            return true;
                        }
                    }
                    _ => panic!("Unreachable!"),
                }
            }
        }

        false
    }
}
