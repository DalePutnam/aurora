use std::fmt;

use na::Vector4;

pub trait Primitive: Send + Sync + fmt::Debug
{
    fn intersect(
        &self,
        ray_origin: &Vector4<f32>,
        ray_direction: &Vector4<f32>,
    ) -> Option<(f32, Vector4<f32>, (f32, f32))>;
    fn get_extents(&self) -> (Vector4<f32>, Vector4<f32>);
}
