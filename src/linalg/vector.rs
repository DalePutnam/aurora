use std::cmp::PartialEq;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Mul;
use std::ops::MulAssign;

use na;

const NUM_ELEMENTS: usize = 3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector
{
    pub(super) vec: na::Vector4<f32>,
}

impl Vector
{
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self
    {
        Vector {
            vec: na::Vector4::new(x, y, z, 0.0),
        }
    }

    #[inline]
    pub fn zeros() -> Self
    {
        Vector {
            vec: na::Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    #[inline]
    pub fn dot(&self, other: &Self) -> f32
    {
        self.vec.dot(&other.vec)
    }

    #[inline]
    pub fn cross(&self, other: &Self) -> Self
    {
        let lhs = self.vec.fixed_rows::<3>(0);
        let rhs = other.vec.fixed_rows::<3>(0);

        Vector {
            vec: lhs.cross(&rhs).insert_row(3, 0.0),
        }
    }

    #[inline]
    pub fn magnitude(&self) -> f32
    {
        self.vec.norm()
    }

    #[inline]
    pub fn normalize(&self) -> Self
    {
        Vector {
            vec: self.vec.normalize(),
        }
    }

    #[inline]
    pub fn normalize_mut(&mut self)
    {
        self.vec.normalize_mut();
    }
}

impl Deref for Vector
{
    type Target = na::base::coordinates::XYZ<f32>;

    #[inline]
    fn deref(&self) -> &Self::Target
    {
        unsafe { &*(self.vec.as_ptr() as *const Self::Target) }
    }
}

impl DerefMut for Vector
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        unsafe { &mut *(self.vec.as_ptr() as *mut Self::Target) }
    }
}

impl Index<usize> for Vector
{
    type Output = f32;

    #[inline]
    fn index(&self, i: usize) -> &Self::Output
    {
        if i >= NUM_ELEMENTS {
            panic!("Vector index out of bounds");
        }

        &self.vec[i]
    }
}

impl IndexMut<usize> for Vector
{
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output
    {
        if i >= NUM_ELEMENTS {
            panic!("Vector index out of bounds");
        }

        &mut self.vec[i]
    }
}

impl Mul<f32> for Vector
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output
    {
        Vector {
            vec: self.vec * rhs,
        }
    }
}

impl Mul<f32> for &Vector
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output
    {
        Mul::mul(*self, rhs)
    }
}

impl Mul<&f32> for Vector
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: &f32) -> Self::Output
    {
        Mul::mul(self, *rhs)
    }
}

impl Mul<&f32> for &Vector
{
    type Output = Vector;

    #[inline]
    fn mul(self, rhs: &f32) -> Self::Output
    {
        Mul::mul(*self, *rhs)
    }
}

impl MulAssign<f32> for Vector
{
    #[inline]
    fn mul_assign(&mut self, rhs: f32)
    {
        self.vec *= rhs;
    }
}

impl MulAssign<&f32> for Vector
{
    #[inline]
    fn mul_assign(&mut self, rhs: &f32)
    {
        MulAssign::mul_assign(self, *rhs)
    }
}

impl MulAssign<f32> for &mut Vector
{
    #[inline]
    fn mul_assign(&mut self, rhs: f32)
    {
        MulAssign::mul_assign(*self, rhs)
    }
}

impl MulAssign<&f32> for &mut Vector
{
    #[inline]
    fn mul_assign(&mut self, rhs: &f32)
    {
        MulAssign::mul_assign(*self, *rhs)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn constructors()
    {
        let zeros = Vector::zeros();

        assert_eq!(zeros.vec[0], 0.0);
        assert_eq!(zeros.vec[1], 0.0);
        assert_eq!(zeros.vec[2], 0.0);

        let vector = Vector::new(1.0, 2.0, 3.0);

        assert_eq!(vector.vec[0], 1.0);
        assert_eq!(vector.vec[1], 2.0);
        assert_eq!(vector.vec[2], 3.0);
    }

    #[test]
    fn components_accessible_by_name()
    {
        let vector = Vector::new(1.0, 2.0, 3.0);

        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 2.0);
        assert_eq!(vector.z, 3.0);

        let mut vector = Vector::zeros();
        vector.x = 4.0;
        vector.y = 5.0;
        vector.z = 6.0;

        assert_eq!(vector.x, 4.0);
        assert_eq!(vector.y, 5.0);
        assert_eq!(vector.z, 6.0);
    }

    #[test]
    fn components_accessible_by_index()
    {
        let vector = Vector::new(1.0, 2.0, 3.0);

        assert_eq!(vector[0], 1.0);
        assert_eq!(vector[1], 2.0);
        assert_eq!(vector[2], 3.0);

        let mut vector = Vector::zeros();
        vector[0] = 4.0;
        vector[1] = 5.0;
        vector[2] = 6.0;

        assert_eq!(vector[0], 4.0);
        assert_eq!(vector[1], 5.0);
        assert_eq!(vector[2], 6.0);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_access()
    {
        let vector = Vector::zeros();
        let _ = vector[3];
    }

    #[test]
    fn mul()
    {
        let a = Vector::new(10.0, 20.0, 30.0);
        let t = 5.0;

        let b = a * t;

        assert_eq!(b, Vector::new(50.0, 100.0, 150.0));

        let a = Vector::new(10.0, 20.0, 30.0);
        let t = &5.0;

        let b = a * t;

        assert_eq!(b, Vector::new(50.0, 100.0, 150.0));

        let a = &Vector::new(10.0, 20.0, 30.0);
        let t = 5.0;

        let b = a * t;

        assert_eq!(b, Vector::new(50.0, 100.0, 150.0));

        let a = &Vector::new(10.0, 20.0, 30.0);
        let t = &5.0;

        let b = a * t;

        assert_eq!(b, Vector::new(50.0, 100.0, 150.0));
    }

    #[test]
    fn mul_assign()
    {
        let mut v = Vector::new(10.0, 20.0, 30.0);
        let t = 5.0;

        v *= t;

        assert_eq!(v, Vector::new(50.0, 100.0, 150.0));

        let mut v = Vector::new(10.0, 20.0, 30.0);
        let t = &5.0;

        v *= t;

        assert_eq!(v, Vector::new(50.0, 100.0, 150.0));

        let mut v = &mut Vector::new(10.0, 20.0, 30.0);
        let t = 5.0;

        v *= t;

        assert_eq!(*v, Vector::new(50.0, 100.0, 150.0));

        let mut v = &mut Vector::new(10.0, 20.0, 30.0);
        let t = &5.0;

        v *= t;

        assert_eq!(*v, Vector::new(50.0, 100.0, 150.0));
    }
}
