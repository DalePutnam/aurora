use linalg::Normal;
use linalg::Point;
use linalg::Transform;
use linalg::Vector;
use na::Vector3;
use primitives::Intersection;
use shading::Material;
use shading::UV;
use Ray;

pub struct Interaction<'a>
{
    material: &'a dyn Material,
    transform: Transform,
    intersect_scalar: f32,
    intersect_point: Point,
    w_out: Vector,
    _uv: UV,
}

impl<'a> Interaction<'a>
{
    pub fn new(intersection: &Intersection, material: &'a dyn Material, ray: &Ray) -> Self
    {
        let vertical = Normal::z_axis();
        let nvertical = -vertical;

        let rotation_axis = if intersection.normal == vertical || intersection.normal == nvertical {
            Normal::x_axis()
        } else {
            Normal::from_vector(&intersection.normal.cross(&vertical))
        };

        let rotation_angle = intersection.normal.dot(&vertical).acos();
        let transform = Transform::from_axis_angle(&rotation_axis, rotation_angle);

        let intersect_point = ray.origin() + (ray.direction() * intersection.t);
        let w_out = transform * -ray.direction().normalize();

        Interaction {
            material: material,
            transform: transform,
            intersect_scalar: intersection.t,
            intersect_point: intersect_point,
            w_out: w_out,
            _uv: intersection.uv,
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
        self.intersect_scalar
    }

    pub fn get_intersection(&self) -> &Point
    {
        &self.intersect_point
    }
}
