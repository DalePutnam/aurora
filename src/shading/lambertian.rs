use std::f32;
use std::fmt;

use linalg::Point;
use linalg::Vector;
use na::Vector3;
use shading::Material;
use util::sampling;

#[derive(fmt::Debug)]
pub struct Lambertian
{
    colour: Vector3<f32>,
}

impl Lambertian
{
    pub fn new(colour: &Vector3<f32>) -> Self
    {
        Lambertian { colour: *colour }
    }
}

impl Material for Lambertian
{
    fn bsdf(&self, _w_in: &Vector, _w_out: &Vector) -> Vector3<f32>
    {
        self.colour / f32::consts::PI
    }

    fn sample_bsdf(&self, w_out: &Vector, u: (f32, f32)) -> (Vector3<f32>, Vector, f32)
    {
        let w_in = sampling::cosine_sample_hemisphere(u) - Point::origin();
        let pdf = self.pdf(&w_in);
        let bdsf = self.bsdf(&w_in, &w_out);

        (bdsf, w_in, pdf)
    }

    fn pdf(&self, w_in: &Vector) -> f32
    {
        sampling::cosine_hemisphere_pdf(w_in.z.abs())
    }
}
