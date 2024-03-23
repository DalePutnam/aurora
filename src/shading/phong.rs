use std::fmt;

use na::Vector3;
use na::Vector4;
use shading::Material;

#[derive(fmt::Debug)]
pub struct Phong
{
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
    shininess: f32,
}

impl Phong
{
    pub fn new(diffuse: Vector3<f32>, specular: Vector3<f32>, shininess: f32) -> Self
    {
        Phong {
            diffuse: diffuse,
            specular: specular,
            shininess: shininess,
        }
    }
}

impl Material for Phong
{
    fn ambient_component(&self) -> Vector3<f32>
    {
        self.diffuse
    }

    fn diffuse_component(&self, light: Vector4<f32>, normal: Vector4<f32>) -> Vector3<f32>
    {
        self.diffuse * light.dot(&normal).max(0.0)
    }

    fn specular_component(
        &self,
        view: Vector4<f32>,
        light: Vector4<f32>,
        normal: Vector4<f32>,
    ) -> Vector3<f32>
    {
        let t = light.dot(&normal) * 2.0;
        let r = normal.map(|component| component * t) - light;

        self.specular * f32::max(r.dot(&view), 0.0).powf(self.shininess)
    }
}
