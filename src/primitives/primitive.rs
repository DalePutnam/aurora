use std::fmt;

use linalg::Normal;
use linalg::Point;
use linalg::Vector;
use shading::UV;

#[derive(Clone, Copy)]
pub struct Intersection
{
    pub t: f32,
    pub normal: Normal,
    pub uv: UV,
}

pub trait Primitive: Send + Sync + fmt::Debug
{
    fn intersect(&self, ray_origin: &Point, ray_direction: &Vector) -> Option<Intersection>;
    fn get_extents(&self) -> (Point, Point);
}
