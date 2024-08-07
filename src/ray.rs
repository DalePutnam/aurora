use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use linalg::Point;
use linalg::Vector;

static NEXT_RAY_ID: AtomicU64 = AtomicU64::new(0);

pub struct Ray
{
    id: u64,
    origin: Point,
    direction: Vector,
}

impl Ray
{
    pub fn new(origin: &Point, direction: &Vector) -> Self
    {
        Ray {
            id: NEXT_RAY_ID.fetch_add(1, Ordering::Relaxed),
            direction: *direction,
            origin: *origin,
        }
    }

    pub fn id(&self) -> u64
    {
        self.id
    }

    pub fn direction(&self) -> &Vector
    {
        &self.direction
    }

    pub fn origin(&self) -> &Point
    {
        &self.origin
    }
}
