use std::cmp::PartialEq;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Mul;

use na;

use super::Normal;
use super::Point;
use super::Vector;

const NUM_ELEMENTS: usize = 16;

#[derive(Clone, Copy, PartialEq)]
pub struct Transform
{
    pub(super) mat: na::Matrix4<f32>,
    inv: na::Matrix4<f32>,
}

impl Transform
{
    #[inline]
    pub fn identity() -> Self
    {
        Transform {
            mat: na::Matrix4::identity(),
            inv: na::Matrix4::identity(),
        }
    }

    #[inline]
    pub fn new(
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m24: f32,
        m31: f32,
        m32: f32,
        m33: f32,
        m34: f32,
        m41: f32,
        m42: f32,
        m43: f32,
        m44: f32,
    ) -> Self
    {
        let mat = na::Matrix4::new(
            m11, m12, m13, m14, m21, m22, m23, m24, m31, m32, m33, m34, m41, m42, m43, m44,
        );

        Transform {
            mat: mat,
            inv: mat.try_inverse().unwrap(),
        }
    }

    #[inline]
    pub fn new_translation(x: f32, y: f32, z: f32) -> Self
    {
        let mat = na::Matrix4::new_translation(&na::Vector3::new(x, y, z));

        Transform {
            mat: mat,
            inv: mat.try_inverse().unwrap(),
        }
    }

    #[inline]
    pub fn new_nonuniform_scaling(x: f32, y: f32, z: f32) -> Self
    {
        let mat = na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(x, y, z));

        Transform {
            mat: mat,
            inv: mat.try_inverse().unwrap(),
        }
    }

    #[inline]
    pub fn from_axis_angle(axis: &Vector, angle: f32) -> Self
    {
        let axis = na::Unit::new_normalize(na::Vector3::new(axis.x, axis.y, axis.z));
        let mat = na::Matrix4::from_axis_angle(&axis, angle);

        Transform {
            mat: mat,
            inv: mat.try_inverse().unwrap(),
        }
    }

    #[inline]
    pub fn inverse(&self) -> TransformRef
    {
        TransformRef {
            mat: &self.inv,
            inv: &self.mat,
        }
    }
}

impl<'a> From<TransformRef<'a>> for Transform
{
    fn from(value: TransformRef<'a>) -> Self
    {
        Transform {
            mat: *value.mat,
            inv: *value.inv,
        }
    }
}

impl Index<usize> for Transform
{
    type Output = f32;

    #[inline]
    fn index(&self, i: usize) -> &Self::Output
    {
        if i >= NUM_ELEMENTS {
            panic!("Transform index out of bounds");
        }

        &self.mat[i]
    }
}

impl IndexMut<usize> for Transform
{
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output
    {
        if i >= NUM_ELEMENTS {
            panic!("Transform index out of bounds");
        }

        &mut self.mat[i]
    }
}

impl Mul<&Transform> for &Transform
{
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: &Transform) -> Self::Output
    {
        let mat = self.mat * rhs.mat;

        Transform {
            mat: mat,
            inv: mat.try_inverse().unwrap(),
        }
    }
}

impl Mul<&Transform> for Transform
{
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: &Transform) -> Self::Output
    {
        Mul::mul(&self, rhs)
    }
}

impl Mul<Transform> for &Transform
{
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: Transform) -> Self::Output
    {
        Mul::mul(self, &rhs)
    }
}

impl Mul<Transform> for Transform
{
    type Output = Transform;

    #[inline]
    fn mul(self, rhs: Transform) -> Self::Output
    {
        Mul::mul(&self, &rhs)
    }
}

impl Mul<&Point> for &Transform
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: &Point) -> Self::Output
    {
        Point {
            vec: self.mat * rhs.vec,
        }
    }
}

impl Mul<&Point> for Transform
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: &Point) -> Self::Output
    {
        Mul::mul(&self, rhs)
    }
}

impl Mul<Point> for &Transform
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: Point) -> Self::Output
    {
        Mul::mul(self, &rhs)
    }
}

impl Mul<Point> for Transform
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: Point) -> Self::Output
    {
        Mul::mul(&self, &rhs)
    }
}

impl Mul<&Vector> for &Transform
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: &Vector) -> Self::Output
    {
        Vector {
            vec: self.mat * rhs.vec,
        }
    }
}

impl Mul<&Vector> for Transform
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: &Vector) -> Self::Output
    {
        Mul::mul(&self, rhs)
    }
}

impl Mul<Vector> for &Transform
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output
    {
        Mul::mul(self, &rhs)
    }
}

impl Mul<Vector> for Transform
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output
    {
        Mul::mul(&self, &rhs)
    }
}

impl Mul<&Normal> for &Transform
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: &Normal) -> Self::Output
    {
        let x = self.inv.m11 * rhs.x + self.inv.m21 * rhs.y + self.inv.m31 * rhs.z;
        let y = self.inv.m12 * rhs.x + self.inv.m22 * rhs.y + self.inv.m32 * rhs.z;
        let z = self.inv.m13 * rhs.x + self.inv.m23 * rhs.y + self.inv.m33 * rhs.z;

        Normal::from_vector(&Vector::new(x, y, z))
    }
}

impl Mul<&Normal> for Transform
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: &Normal) -> Self::Output
    {
        Mul::mul(&self, rhs)
    }
}

impl Mul<Normal> for &Transform
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: Normal) -> Self::Output
    {
        Mul::mul(self, &rhs)
    }
}

impl Mul<Normal> for Transform
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: Normal) -> Self::Output
    {
        Mul::mul(&self, &rhs)
    }
}

pub struct TransformRef<'a>
{
    pub(super) mat: &'a na::Matrix4<f32>,
    inv: &'a na::Matrix4<f32>,
}

impl<'a> Index<usize> for TransformRef<'a>
{
    type Output = f32;

    #[inline]
    fn index(&self, i: usize) -> &Self::Output
    {
        if i >= NUM_ELEMENTS {
            panic!("Transform index out of bounds");
        }

        &self.mat[i]
    }
}

impl<'a> Mul<&Point> for &TransformRef<'a>
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: &Point) -> Self::Output
    {
        Point {
            vec: self.mat * rhs.vec,
        }
    }
}

impl<'a> Mul<&Point> for TransformRef<'a>
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: &Point) -> Self::Output
    {
        Mul::mul(&self, rhs)
    }
}

impl<'a> Mul<Point> for &TransformRef<'a>
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: Point) -> Self::Output
    {
        Mul::mul(self, &rhs)
    }
}

impl<'a> Mul<Point> for TransformRef<'a>
{
    type Output = Point;

    #[inline]
    fn mul(self, rhs: Point) -> Self::Output
    {
        Mul::mul(&self, &rhs)
    }
}

impl<'a> Mul<&Vector> for &TransformRef<'a>
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: &Vector) -> Self::Output
    {
        Vector {
            vec: self.mat * rhs.vec,
        }
    }
}

impl<'a> Mul<&Vector> for TransformRef<'a>
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: &Vector) -> Self::Output
    {
        Mul::mul(&self, rhs)
    }
}

impl<'a> Mul<Vector> for &TransformRef<'a>
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output
    {
        Mul::mul(self, &rhs)
    }
}

impl<'a> Mul<Vector> for TransformRef<'a>
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: Vector) -> Self::Output
    {
        Mul::mul(&self, &rhs)
    }
}

impl<'a> Mul<&Normal> for &TransformRef<'a>
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: &Normal) -> Self::Output
    {
        let x = self.inv.m11 * rhs.x + self.inv.m21 * rhs.y + self.inv.m31 * rhs.z;
        let y = self.inv.m12 * rhs.x + self.inv.m22 * rhs.y + self.inv.m32 * rhs.z;
        let z = self.inv.m13 * rhs.x + self.inv.m23 * rhs.y + self.inv.m33 * rhs.z;

        Normal::from_vector(&Vector::new(x, y, z))
    }
}

impl<'a> Mul<&Normal> for TransformRef<'a>
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: &Normal) -> Self::Output
    {
        Mul::mul(&self, rhs)
    }
}

impl<'a> Mul<Normal> for &TransformRef<'a>
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: Normal) -> Self::Output
    {
        Mul::mul(self, &rhs)
    }
}

impl<'a> Mul<Normal> for TransformRef<'a>
{
    type Output = Normal;

    #[inline]
    fn mul(self, rhs: Normal) -> Self::Output
    {
        Mul::mul(&self, &rhs)
    }
}

#[cfg(test)]
mod tests
{
    use std::f32;

    use super::*;

    #[test]
    fn constructors()
    {
        let identity = Transform::identity();

        assert_eq!(identity.mat[0], 1.0);
        assert_eq!(identity.mat[1], 0.0);
        assert_eq!(identity.mat[2], 0.0);
        assert_eq!(identity.mat[3], 0.0);
        assert_eq!(identity.mat[4], 0.0);
        assert_eq!(identity.mat[5], 1.0);
        assert_eq!(identity.mat[6], 0.0);
        assert_eq!(identity.mat[7], 0.0);
        assert_eq!(identity.mat[8], 0.0);
        assert_eq!(identity.mat[9], 0.0);
        assert_eq!(identity.mat[10], 1.0);
        assert_eq!(identity.mat[11], 0.0);
        assert_eq!(identity.mat[12], 0.0);
        assert_eq!(identity.mat[13], 0.0);
        assert_eq!(identity.mat[14], 0.0);
        assert_eq!(identity.mat[15], 1.0);

        let transform = Transform::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 3.0, 4.0, 1.0,
        );

        assert_eq!(transform.mat[0], 1.0);
        assert_eq!(transform.mat[1], 0.0);
        assert_eq!(transform.mat[2], 0.0);
        assert_eq!(transform.mat[3], 2.0);
        assert_eq!(transform.mat[4], 0.0);
        assert_eq!(transform.mat[5], 1.0);
        assert_eq!(transform.mat[6], 0.0);
        assert_eq!(transform.mat[7], 3.0);
        assert_eq!(transform.mat[8], 0.0);
        assert_eq!(transform.mat[9], 0.0);
        assert_eq!(transform.mat[10], 1.0);
        assert_eq!(transform.mat[11], 4.0);
        assert_eq!(transform.mat[12], 0.0);
        assert_eq!(transform.mat[13], 0.0);
        assert_eq!(transform.mat[14], 0.0);
        assert_eq!(transform.mat[15], 1.0);

        let t2: Transform = transform.inverse().into();
    }

    #[test]
    fn rotation_matrix()
    {
        let n = Normal::from_vector(&Vector::new(1.0, 0.0, 0.0));
        let v = Vector::new(0.0, 1.0, 0.0);

        let t = Transform::from_axis_angle(&n, f32::consts::PI / 2.0);
        let v = t * v;

        assert_eq!(v.x, 0.0);
        assert!(v.y < 0.0000001); // Fuzzy compare
        assert_eq!(v.z, 1.0);
    }
}
