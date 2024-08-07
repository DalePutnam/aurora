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
pub struct Cube
{
    position: Point,
    size: f32,
}

impl Cube
{
    pub fn unit_cube() -> Self
    {
        Cube {
            position: Point::origin(),
            size: 1.0,
        }
    }

    pub fn new(position: Vector3<f32>, size: f32) -> Self
    {
        Cube {
            position: Point::new(position.x, position.y, position.z),
            size: size,
        }
    }
}

impl Primitive for Cube
{
    //fn hit(&self, ray: &Ray, transform: Matrix4<f32>) -> Option<Hit>
    fn intersect(&self, ray_origin: &Point, ray_direction: &Vector) -> Option<Intersection>
    {
        enum Faces
        {
            Front,
            Back,
            Top,
            Bottom,
            Left,
            Right,
        }

        let inv_direction_x = 1.0 / ray_direction.x;
        let inv_direction_y = 1.0 / ray_direction.y;
        let inv_direction_z = 1.0 / ray_direction.z;

        let min = (self.position.x - ray_origin.x) * inv_direction_x;
        let max = (self.position.x + self.size - ray_origin.x) * inv_direction_x;

        let (mut t_min, mut face_min, mut t_max, mut face_max) = if inv_direction_x >= 0.0 {
            (min, Faces::Left, max, Faces::Right)
        } else {
            (max, Faces::Right, min, Faces::Left)
        };

        let min = (self.position.y - ray_origin.y) * inv_direction_y;
        let max = (self.position.y + self.size - ray_origin.y) * inv_direction_y;

        let (ty_min, y_min_face, ty_max, y_max_face) = if inv_direction_y >= 0.0 {
            (min, Faces::Bottom, max, Faces::Top)
        } else {
            (max, Faces::Top, min, Faces::Bottom)
        };

        if (t_min > ty_max) || (ty_min > t_max) {
            return None;
        }

        if ty_min > t_min {
            t_min = ty_min;
            face_min = y_min_face;
        }

        if ty_max < t_max {
            t_max = ty_max;
            face_max = y_max_face;
        }

        let min = (self.position.z - ray_origin.z) * inv_direction_z;
        let max = (self.position.z + self.size - ray_origin.z) * inv_direction_z;

        let (tz_min, z_face_min, tz_max, z_face_max) = if inv_direction_z >= 0.0 {
            (min, Faces::Back, max, Faces::Front)
        } else {
            (max, Faces::Front, min, Faces::Back)
        };

        if (t_min > tz_max) || (tz_min > t_max) {
            return None;
        }

        if tz_min > t_min {
            t_min = tz_min;
            face_min = z_face_min;
        }

        if tz_max < t_max {
            t_max = tz_max;
            face_max = z_face_max;
        }

        let (intersect, face) = if math::far_from_zero_pos(t_min) {
            (t_min, face_min)
        } else if math::far_from_zero_pos(t_max) {
            (t_max, face_max)
        } else {
            return None;
        };

        let normal = match face {
            Faces::Right => Normal::x_axis(),
            Faces::Left => -Normal::x_axis(),
            Faces::Top => Normal::y_axis(),
            Faces::Bottom => -Normal::y_axis(),
            Faces::Front => Normal::z_axis(),
            Faces::Back => -Normal::z_axis(),
        };

        //let world_normal = math::transform_normals(local_normal, transform);
        // TODO: UV value calculation

        // Some(Hit {
        //     intersect: intersect,
        //     normal: world_normal,
        //     uv: (0.0, 0.0),
        // })
        Some(Intersection {
            t: intersect,
            normal: normal,
            uv: UV(0.0, 0.0),
        })
    }

    fn get_extents(&self) -> (Point, Point)
    {
        (self.position, self.position + Vector::repeat(self.size))
    }
}
