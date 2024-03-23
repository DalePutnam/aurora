use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use na::Vector4;

static NEXT_RAY_ID: AtomicU64 = AtomicU64::new(0);

pub struct Ray
{
    id: u64,
    point: Vector4<f32>,
    origin: Vector4<f32>,
}

impl Ray
{
    pub fn new(origin: Vector4<f32>, point: Vector4<f32>) -> Self
    {
        Ray {
            id: NEXT_RAY_ID.fetch_add(1, Ordering::Relaxed),
            point: point,
            origin: origin,
        }
    }

    pub fn new2(origin: &Vector4<f32>, direction: &Vector4<f32>) -> Self
    {
        Ray {
            id: NEXT_RAY_ID.fetch_add(1, Ordering::Relaxed),
            point: origin + direction,
            origin: *origin,
        }
    }

    pub fn id(&self) -> u64
    {
        self.id
    }

    pub fn point(&self) -> Vector4<f32>
    {
        self.point
    }

    pub fn origin(&self) -> Vector4<f32>
    {
        self.origin
    }
}

pub struct Hit
{
    pub intersect: f32,
    pub normal: Vector4<f32>,
    pub uv: (f32, f32),
}
