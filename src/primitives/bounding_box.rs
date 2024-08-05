use std::fmt;

use linalg::Point;
use linalg::Transform;
use util::math;
use Ray;

#[derive(fmt::Debug)]
pub struct BoundingBox
{
    lower_point: Point,
    upper_point: Point,
}

impl BoundingBox
{
    pub fn new(lower_point: &Point, upper_point: &Point) -> Self
    {
        BoundingBox {
            lower_point: *lower_point,
            upper_point: *upper_point,
        }
    }

    pub fn get_extents(&self) -> (Point, Point)
    {
        (self.lower_point, self.upper_point)
    }

    pub fn hit(&self, ray: &Ray, transform: &Transform) -> bool
    {
        let direction = transform * ray.direction();
        let origin = transform * ray.origin();

        let inv_direction_x = 1.0 / direction.x;
        let inv_direction_y = 1.0 / direction.y;
        let inv_direction_z = 1.0 / direction.z;

        let min = (self.lower_point.x - origin.x) * inv_direction_x;
        let max = (self.upper_point.x - origin.x) * inv_direction_x;

        let (mut t_min, mut t_max) = if inv_direction_x >= 0.0 {
            (min, max)
        } else {
            (max, min)
        };

        let min = (self.lower_point.y - origin.y) * inv_direction_y;
        let max = (self.upper_point.y - origin.y) * inv_direction_y;

        let (ty_min, ty_max) = if inv_direction_y >= 0.0 {
            (min, max)
        } else {
            (max, min)
        };

        if (t_min > ty_max) || (ty_min > t_max) {
            return false;
        }

        if ty_min > t_min {
            t_min = ty_min;
        }

        if ty_max < t_max {
            t_max = ty_max;
        }

        let min = (self.lower_point.z - origin.z) * inv_direction_z;
        let max = (self.upper_point.z - origin.z) * inv_direction_z;

        let (tz_min, tz_max) = if inv_direction_z >= 0.0 {
            (min, max)
        } else {
            (max, min)
        };

        if (t_min > tz_max) || (tz_min > t_max) {
            return false;
        }

        if tz_min > t_min {
            t_min = tz_min;
        }

        if tz_max < t_max {
            t_max = tz_max;
        }

        if !math::far_from_zero_pos(t_min) && !math::far_from_zero_pos(t_max) {
            return false;
        }

        true
    }
}
