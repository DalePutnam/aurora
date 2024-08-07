use std::f32;
use std::fmt;

use linalg::Normal;
use linalg::Point;
use linalg::Vector;
use na::Vector3;
use primitives::Intersection;
use primitives::Primitive;
use shading::UV;
use util::math;

#[derive(fmt::Debug)]
pub struct Sphere
{
    position: Point,
    radius: f32,
}

impl Sphere
{
    pub fn unit_sphere() -> Sphere
    {
        Sphere {
            position: Point::origin(),
            radius: 1.0,
        }
    }

    pub fn new(position: Vector3<f32>, radius: f32) -> Self
    {
        Sphere {
            position: Point::new(position.x, position.y, position.z),
            radius: radius,
        }
    }
}

impl Primitive for Sphere
{
    fn intersect(&self, ray_origin: &Point, ray_direction: &Vector) -> Option<Intersection>
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

                    let mut normal = Normal::from_vector(
                        &((ray_origin + (ray_direction * intersect)) - self.position),
                    );

                    // Invert normal if inside sphere
                    if normal.dot(&(-ray_direction)) < 0.0 {
                        normal = -normal;
                    }

                    Some(Intersection {
                        t: intersect,
                        normal: normal,
                        uv: UV(0.0, 0.0),
                    })
                }
            },
        }
    }

    fn get_extents(&self) -> (Point, Point)
    {
        (
            self.position + Vector::repeat(-self.radius),
            self.position + Vector::repeat(self.radius),
        )
    }
}
