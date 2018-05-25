use na::Vector4;

pub struct Ray {
    pub id: u32,
    pub thread_id: u32,
    pub point: Vector4<f32>,
    pub origin: Vector4<f32>,
}

pub struct Hit {
    pub intersect: f32,
    pub normal: Vector4<f32>,
    pub uv: (f32, f32),
}

impl Hit {
    pub fn new(intersect: f32, normal: Vector4<f32>, u: f32, v: f32) -> Self {
        Hit {
            intersect: intersect,
            normal: normal,
            uv: (u, v),
        }
    }
}