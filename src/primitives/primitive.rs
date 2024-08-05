use std::fmt;

use linalg::Normal;
use linalg::Point;
use linalg::Vector;

pub trait Primitive: Send + Sync + fmt::Debug
{
    fn intersect(
        &self,
        ray_origin: &Point,
        ray_direction: &Vector,
    ) -> Option<(f32, Normal, (f32, f32))>;
    fn get_extents(&self) -> (Point, Point);
}
