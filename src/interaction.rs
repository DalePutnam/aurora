use na::Matrix4;
use na::Unit;
use na::Vector3;
use na::Vector4;
use na::U3;
use shading::Material;
use util::math;

pub struct Interaction<'a>
{
    material: &'a dyn Material,
    transform: Matrix4<f32>,
    intersect: f32,
    intersection: Vector4<f32>,
    w_out: Vector4<f32>,
    _tex_coords: (f32, f32),
}

impl<'a> Interaction<'a>
{
    pub fn new(
        material: &'a dyn Material,
        intersect: f32,
        intersection: &Vector4<f32>,
        normal: &Vector4<f32>,
        w_out: &Vector4<f32>,
        tex_coords: (f32, f32),
    ) -> Self
    {
        let normal = normal.normalize();
        let w_out = w_out.normalize();

        let vertical = Vector4::new(0.0, 0.0, 1.0, 0.0);
        let nvertical = -vertical;

        let rotation_axis = if normal == vertical || normal == nvertical {
            Vector4::new(1.0, 0.0, 0.0, 0.0)
        } else {
            math::cross_4d(normal, vertical)
        };

        let rotation_angle = normal.dot(&vertical).acos();
        let transform = Matrix4::from_axis_angle(
            &Unit::new_normalize(rotation_axis.fixed_rows::<U3>(0).into()),
            rotation_angle,
        );

        Interaction {
            material: material,
            transform: transform,
            intersect: intersect,
            intersection: *intersection,
            w_out: transform * w_out,
            _tex_coords: tex_coords,
        }
    }

    pub fn sample_bsdf(&self, randomness: (f32, f32)) -> (Vector3<f32>, Vector4<f32>, f32)
    {
        let (bsdf, w_in, pdf) = self.material.sample_bsdf(&self.w_out, randomness);

        let bsdf = bsdf * w_in.z.abs();
        let w_in = self.transform.try_inverse().unwrap() * w_in;

        (bsdf, w_in, pdf)
    }

    pub fn evaluate_bsdf(&self, w_in: &Vector4<f32>) -> Vector3<f32>
    {
        let w_in = self.transform * w_in;
        self.material.bsdf(&w_in, &self.w_out) * w_in.z.abs()
    }

    pub fn get_intersect_scalar(&self) -> f32
    {
        self.intersect
    }

    pub fn get_intersection(&self) -> &Vector4<f32>
    {
        &self.intersection
    }
}
