use na::Matrix4;
use na::Vector4;
use na::U3;

const EPSILON: f32 = 0.0001;

#[inline(always)]
pub fn cross_4d(a: &Vector4<f32>, b: &Vector4<f32>) -> Vector4<f32>
{
	let a3 = a.fixed_rows::<U3>(0);
	let b3 = b.fixed_rows::<U3>(0);

	let c3 = a3.cross(&b3);

	c3.insert_row(3, 0.0)
}

#[inline(always)]
pub fn transform_normals(normal: &Vector4<f32>, transform: &Matrix4<f32>) -> Vector4<f32>
{
	let n3 = normal.fixed_rows::<U3>(0);
	let t33 = transform.fixed_slice::<U3, U3>(0, 0);

	let ln3 = t33.transpose() * n3;

	ln3.insert_row(3, 0.0)
}

#[inline(always)]
pub fn near_zero(number: f32) -> bool
{
	f32::abs(number) < EPSILON
}

#[inline(always)]
pub fn far_from_zero_pos(number: f32) -> bool
{
	number > EPSILON
}

#[inline(always)]
pub fn far_from_zero_neg(number: f32) -> bool
{
	number < -EPSILON
}

pub enum QuadRoots
{
	Two(f32, f32),
	One(f32),
	Zero,
}

pub fn quadratic_roots(a: f32, b: f32, c: f32) -> QuadRoots
{
	if a == 0.0 {
		if b == 0.0 {
			QuadRoots::Zero
		} else {
			QuadRoots::One(-c / b)
		}
	} else {
		let d = b * b - 4.0 * a * c;

		if d < 0.0 {
			QuadRoots::Zero
		} else {
			let sign_b = if b < 0.0 { -1.0 } else { 1.0 };
			let q = -(b + sign_b * d.sqrt()) / 2.0;

			let root_one = q / a;
			let root_two = if q != 0.0 { c / q } else { root_one };

			QuadRoots::Two(root_one, root_two)
		}
	}
}
