static PI: f32 = 3.14159265;

pub fn degrees_to_radians(angle: f32) -> f32 {
    angle * (PI / 180.0)
}