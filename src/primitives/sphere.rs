use std::f32;
use std::fmt;

use na::Vector3;
use na::Vector4;
use primitives::Primitive;
use util::math;

#[derive(fmt::Debug)]
pub struct Sphere
{
    position: Vector4<f32>,
    radius: f32,
}

impl Sphere
{
    pub fn unit_sphere() -> Sphere
    {
        Sphere {
            position: Vector4::new(0.0, 0.0, 0.0, 1.0),
            radius: 1.0,
        }
    }

    pub fn new(position: Vector3<f32>, radius: f32) -> Self
    {
        Sphere {
            position: Vector4::new(position.x, position.y, position.z, 1.0),
            radius: radius,
        }
    }
}

impl Primitive for Sphere
{
    fn intersect(
        &self,
        ray_origin: &Vector4<f32>,
        ray_direction: &Vector4<f32>,
    ) -> Option<(f32, Vector4<f32>, (f32, f32))>
    {
        let oc = ray_origin - self.position;

        let a = ray_direction.dot(&ray_direction);
        let b = ray_direction.dot(&oc) * 2.0;
        let c = oc.dot(&oc) - (self.radius * self.radius);

        match math::quadratic_roots(a, b, c) {
            math::QuadRoots::Zero | math::QuadRoots::One(_) => None,
            math::QuadRoots::Two(root_one, root_two) => {
                if !math::far_from_zero_pos(root_one) && !math::far_from_zero_pos(root_two) {
                    None
                } else {
                    let intersect = if root_one <= root_two {
                        if math::far_from_zero_pos(root_one) {
                            root_one
                        } else {
                            root_two
                        }
                    } else {
                        if math::far_from_zero_pos(root_two) {
                            root_two
                        } else {
                            root_one
                        }
                    };

                    let mut normal = (ray_origin + (intersect * ray_direction)) - self.position;

                    // Invert normal if inside sphere
                    if normal.dot(&(-ray_direction)) < 0.0 {
                        normal = -normal;
                    }

                    Some((intersect, normal, (0.0, 0.0)))
                }
            },
        }
    }

    fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>)
    {
        (
            self.position.add_scalar(-self.radius),
            self.position.add_scalar(self.radius),
        )
    }
}
