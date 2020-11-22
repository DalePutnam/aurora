use na::{Vector4, U3};

pub const EPSILON: f32 = 0.0001;
pub const PI: f32 = 3.14159265;

pub enum QuadRoots {
    Two(f32, f32),
    One(f32),
    Zero,
}

pub fn cross_4d(a: &Vector4<f32>, b: &Vector4<f32>) -> Vector4<f32> {
    let a3 = a.fixed_rows::<U3>(0);
    let b3 = b.fixed_rows::<U3>(0);

    let c3 = a3.cross(&b3);

    c3.insert_row(3, 0.0)
}

pub fn quadratic_roots(a: f32, b: f32, c: f32) -> QuadRoots {
    if a == 0.0 {
        if b == 0.0 {
            QuadRoots::Zero
        } else {
            QuadRoots::One(-c/b)
        }
    } else {
        let d = b*b - 4.0*a*c;

        if d < 0.0 {
            QuadRoots::Zero
        } else {
            let sign_b = if b < 0.0 { -1.0 } else { 0.0 };
            let q = -(b + sign_b*d.sqrt()) / 2.0;

            let root_one = q / a;
            let root_two = if q != 0.0 {
                c / q
            } else {
                root_one
            };

            QuadRoots::Two(root_one, root_two)
        }
    }
}