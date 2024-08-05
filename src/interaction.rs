use linalg::Normal;
use linalg::Point;
use linalg::Transform;
use linalg::Vector;
use na::Vector3;
use shading::Material;

pub struct Interaction<'a>
{
    material: &'a dyn Material,
    transform: Transform,
    intersect: f32,
    intersection: Point,
    w_out: Vector,
    _tex_coords: (f32, f32),
}

impl<'a> Interaction<'a>
{
    pub fn new(
        material: &'a dyn Material,
        intersect: f32,
        intersection: &Point,
        normal: &Normal,
        w_out: &Vector,
        tex_coords: (f32, f32),
    ) -> Self
    {
        let w_out = w_out.normalize();

        let vertical = Normal::z_axis();
        let nvertical = -vertical;

        let rotation_axis = if *normal == vertical || *normal == nvertical {
            Normal::x_axis()
        } else {
            Normal::from_vector(&normal.cross(&vertical))
        };

        let rotation_angle = normal.dot(&vertical).acos();
        let transform = Transform::from_axis_angle(&rotation_axis, rotation_angle);

        Interaction {
            material: material,
            transform: transform,
            intersect: intersect,
            intersection: *intersection,
            w_out: transform * w_out,
            _tex_coords: tex_coords,
        }
    }

    pub fn sample_bsdf(&self, randomness: (f32, f32)) -> (Vector3<f32>, Vector, f32)
    {
        let (bsdf, w_in, pdf) = self.material.sample_bsdf(&self.w_out, randomness);

        let bsdf = bsdf * w_in.z.abs();
        let w_in = self.transform.inverse() * w_in;

        (bsdf, w_in, pdf)
    }

    pub fn evaluate_bsdf(&self, w_in: &Vector) -> Vector3<f32>
    {
        let w_in = self.transform * w_in;
        self.material.bsdf(&w_in, &self.w_out) * w_in.z.abs()
    }

    pub fn get_intersect_scalar(&self) -> f32
    {
        self.intersect
    }

    pub fn get_intersection(&self) -> &Point
    {
        &self.intersection
    }
}
