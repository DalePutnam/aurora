pub use self::primitives::NonhierBox;
pub use self::primitives::NonhierSphere;
pub use self::primitives::Sphere;
pub use self::primitives::Cube;
pub use self::material::Material;
pub use self::ray::Ray;
pub use self::ray::Hit;
pub use self::light::Light;
pub use self::object::Object;
pub use self::mesh::Mesh;
pub use self::bounding_box::BoundingBox;

pub mod traits;
pub mod util;
pub mod primitives;
pub mod material;
pub mod ray;
pub mod object;
pub mod light;
pub mod mesh;
pub mod bounding_box;

use std::sync::Arc;
use na::{Vector3, Vector4, Matrix4};
use std::f32;
use std::fs::File;
use image::{ImageBuffer, Pixel, Rgb, ColorType, png};

pub fn render(objects: Vec<Arc<Object>>, output_name: String, output_width: u32, output_height: u32,
              eye: Vector3<f32>, view: Vector3<f32>, up: Vector3<f32>, fov_y: f32, ambient: Vector3<f32>, lights: Vec<Arc<Light>>)
{
    println!("Aurora Ray Tracer");
    println!("Rendering to {}", output_name);
    println!("Width: {} Height: {}", output_width, output_height);
    println!("Vertical FOV: {}", fov_y);
    println!("Eye:  {{ x: {}, y: {}, z: {} }}", eye.x, eye.y, eye.z);
    println!("View: {{ x: {}, y: {}, z: {} }}", view.x, view.y, view.z);
    println!("Up:   {{ x: {}, y: {}, z: {} }}", up.x, up.y, up.z);
    println!("Rendering {} objects with {} lights", objects.len(), lights.len());

    let nx = output_width as f32;
    let ny = output_height as f32;

    let hi = 2.0;
    let wi = (nx * hi) / ny;
    let d = hi / (2.0 * (fov_y / 2.0).to_radians().tan());

    let w = (view - eye).normalize();
    let u = up.cross(&w).normalize();
    let v = u.cross(&w);

    let t1 = Matrix4::new_translation(&Vector3::new(-nx / 2.0, -ny / 2.0, d));
    let s2 = Matrix4::new_nonuniform_scaling(&Vector3::new(-hi / ny, wi / nx, 1.0));
    let r3 = Matrix4::new(u.x, v.x, w.x, 0.0,
                          u.y, v.y, w.y, 0.0,
                          u.z, v.z, w.z, 0.0,
                          0.0, 0.0, 0.0, 1.0);
    let t4 = Matrix4::new(1.0, 0.0, 0.0, eye.x,
                          0.0, 1.0, 0.0, eye.y,
                          0.0, 0.0, 1.0, eye.z,
                          0.0, 0.0, 0.0, 1.0);

    let stw = t4 * r3 * s2 * t1;

    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(output_width, output_height);
    let eye_4d = Vector4::new(eye.x, eye.y, eye.z, 1.0);

    for x in 0..output_width {
        for y in 0..output_height {
            let pworld = stw * Vector4::new(x as f32, y as f32, 0.0, 1.0);
            let ray = Ray { point: pworld, origin: eye_4d, id: 0, thread_id: 0 };

            let colour = trace_pixel(&ray, &objects, &lights, &ambient);

            let r = (255.0 * colour.x.min(1.0)) as u8;
            let g = (255.0 * colour.y.min(1.0)) as u8;
            let b = (255.0 * colour.z.min(1.0)) as u8;

            image.put_pixel(x, y, *Rgb::from_slice(&[r, g, b]));
        }
    }

    match File::create(&output_name) {
        Ok(file) => {
            let encoder = png::PNGEncoder::new(file);
            match encoder.encode(&image.into_raw(), output_width, output_width, ColorType::RGB(8)) {
                Ok(_) => (),
                Err(e) => println!("ERROR: Unable to encode image: {}", e),
            }
        },
        Err(e) => println!("ERROR: Unable to write to file {}: {}", output_name, e),
    }
}

fn trace_pixel(ray: &Ray, objects: &Vec<Arc<Object>>, lights: &Vec<Arc<Light>>, ambient: &Vector3<f32>) -> Vector3<f32> {
    match check_hit(ray, objects) {
        Some((hit, material)) => {
            let kd = material.get_diffuse();
            let ks = material.get_specular();
            let shininess = material.get_shininess();

            let ac = kd.component_mul(ambient);
            let mut dc = Vector3::new(0.0, 0.0, 0.0);
            let mut sc = Vector3::new(0.0, 0.0, 0.0);

            let contact_point = ray.origin + (hit.intersect * (ray.point - ray.origin));

            for light in lights {
                let shadow_ray = Ray { origin: contact_point, point: *light.get_position(), id: 0, thread_id: 0 };

                if let Some((shadow_hit, _)) = check_hit(&shadow_ray, objects) {
                    if shadow_hit.intersect <= 1.0 {
                        continue;
                    }
                }

                if kd.x != 0.0 || kd.y != 0.0 || kd.z != 0.0 {
                    dc += calculate_diffuse(&contact_point, &hit.normal, kd, &light);
                }

                if ks.x != 0.0 || ks.y != 0.0 || ks.z != 0.0 {
                    sc += calculate_specular(&contact_point, &ray.origin, &hit.normal, ks, shininess, &light);
                }
            }

            ac + dc + sc
        },
        None => {
            Vector3::new(0.0, 0.0, 0.0)
        }
    }
}

fn check_hit(ray: &Ray, objects: &Vec<Arc<Object>>) -> Option<(Hit, Arc<Box<Material>>)> {
    let mut min_intersect = f32::INFINITY;
    let mut final_hit: Option<(Hit, Arc<Box<Material>>)> = None;
    
    for object in objects {
        if let Some((hit, material)) = object.check_hit(ray) {
            if hit.intersect < min_intersect {
                min_intersect = hit.intersect;
                final_hit = Some((hit, material));
            }
        }
    }

    final_hit
}

fn calculate_diffuse(contact_point: &Vector4<f32>, normal: &Vector4<f32>, kd: &Vector3<f32>, light: &Arc<Light>) -> Vector3<f32> {
    let light_vector = light.get_position() - contact_point;
    let distance = light_vector.dot(&light_vector).sqrt();

    let factor = light_vector.normalize().dot(&normal.normalize()).max(0.0);

    attenuate_light(distance, light).component_mul(kd).component_mul(&Vector3::repeat(factor))
}

fn calculate_specular(contact_point: &Vector4<f32>, eye: &Vector4<f32>, normal: &Vector4<f32>, ks: &Vector3<f32>, shininess: f32, light: &Arc<Light>) -> Vector3<f32> {
    let light_vector = light.get_position() - contact_point;
    let distance = light_vector.dot(&light_vector).sqrt();

    let v = (eye - contact_point).normalize();
    let l = light_vector.normalize();
    let n = normal.normalize();

    let t = l.dot(&n) * 2.0;
    let r = n.map(|component| { component * t }) - l;
    let factor = r.dot(&v).max(0.0).powf(shininess);


    attenuate_light(distance, light).component_mul(ks).component_mul(&Vector3::repeat(factor))
}

fn attenuate_light(distance: f32, light: &Arc<Light>) -> Vector3<f32> {
    let c = light.get_falloff();
    light.get_colour() / (c.x + (c.y * distance) + (c.z * distance * distance))
}
