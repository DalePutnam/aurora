use std::f32;

use na::Vector4;

pub fn concentric_sample_disk(mut u: (f32, f32)) -> Vector4<f32>
{
    u.0 = 2.0 * u.0 - 1.0;
    u.1 = 2.0 * u.1 - 1.0;

    if u.0 == 0. && u.1 == 0. {
        return Vector4::new(0.0, 0.0, 0.0, 0.0);
    }

    let (r, theta) = if u.0.abs() > u.1.abs() {
        (u.0, (std::f32::consts::PI / 4.0) * (u.1 / u.0))
    } else {
        (
            u.1,
            (std::f32::consts::PI / 2.0) - (std::f32::consts::PI / 4.0) * (u.0 / u.1),
        )
    };

    Vector4::new(theta.cos(), theta.sin(), 0.0, 0.0) * r
}

pub fn cosine_sample_hemisphere(u: (f32, f32)) -> Vector4<f32>
{
    let disk_point = concentric_sample_disk(u);
    let z = 0.0_f32
        .max(1.0 - disk_point.x * disk_point.x - disk_point.y * disk_point.y)
        .sqrt();

    Vector4::new(disk_point.x, disk_point.y, z, 0.0)
}

pub fn cosine_hemisphere_pdf(cos_theta: f32) -> f32
{
    cos_theta / f32::consts::PI
}
