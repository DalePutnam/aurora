use na::Vector4;
use std::sync::{Arc, RwLock};
use core::nodes::SceneNode;

pub struct Ray {
    pub id: u32,
    pub thread_id: u32,
    pub point: Vector4<f32>,
    pub origin: Vector4<f32>,
}

pub struct Hit {
    intersect: f32,
    normal: Vector4<f32>,
    uv: (f32, f32),
    node: Arc<RwLock<SceneNode>>,
}