use std::cmp::PartialEq;
use std::fmt::Debug;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Sub;

use na;

use super::Vector;

const NUM_ELEMENTS: usize = 3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point
{
    pub(super) vec: na::Vector4<f32>,
}

impl Point
{
    pub fn new(x: f32, y: f32, z: f32) -> Self
    {
        Point {
            vec: na::Vector4::new(x, y, z, 1.0),
        }
    }

    pub fn origin() -> Self
    {
        Point {
            vec: na::Vector4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

impl Deref for Point
{
    type Target = na::base::coordinates::XYZ<f32>;

    #[inline]
    fn deref(&self) -> &Self::Target
    {
        unsafe { &*(self.vec.as_ptr() as *const Self::Target) }
    }
}

impl DerefMut for Point
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        unsafe { &mut *(self.vec.as_ptr() as *mut Self::Target) }
    }
}

impl Index<usize> for Point
{
    type Output = f32;

    #[inline]
    fn index(&self, i: usize) -> &Self::Output
    {
        if i >= NUM_ELEMENTS {
            panic!("Point index out of bounds");
        }

        &self.vec[i]
    }
}

impl IndexMut<usize> for Point
{
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output
    {
        if i >= NUM_ELEMENTS {
            panic!("Point index out of bounds");
        }

        &mut self.vec[i]
    }
}

impl Sub<Point> for Point
{
    type Output = Vector;

    #[inline]
    fn sub(self, rhs: Point) -> Self::Output
    {
        Vector {
            vec: self.vec - rhs.vec,
        }
    }
}

impl Sub<&Point> for Point
{
    type Output = Vector;

    #[inline]
    fn sub(self, rhs: &Point) -> Self::Output
    {
        Sub::sub(self, *rhs)
    }
}

impl Sub<Point> for &Point
{
    type Output = Vector;

    #[inline]
    fn sub(self, rhs: Point) -> Self::Output
    {
        Sub::sub(*self, rhs)
    }
}

impl Sub<&Point> for &Point
{
    type Output = Vector;

    #[inline]
    fn sub(self, rhs: &Point) -> Self::Output
    {
        Sub::sub(*self, *rhs)
    }
}

impl Add<Vector> for Point
{
    type Output = Point;

    #[inline]
    fn add(self, rhs: Vector) -> Self::Output
    {
        Point {
            vec: self.vec + rhs.vec,
        }
    }
}

impl Add<&Vector> for Point
{
    type Output = Point;

    #[inline]
    fn add(self, rhs: &Vector) -> Self::Output
    {
        Add::add(self, *rhs)
    }
}

impl Add<Vector> for &Point
{
    type Output = Point;

    #[inline]
    fn add(self, rhs: Vector) -> Self::Output
    {
        Add::add(*self, rhs)
    }
}

impl Add<&Vector> for &Point
{
    type Output = Point;

    #[inline]
    fn add(self, rhs: &Vector) -> Self::Output
    {
        Add::add(*self, *rhs)
    }
}

impl AddAssign<Vector> for Point
{
    #[inline]
    fn add_assign(&mut self, rhs: Vector)
    {
        self.vec += rhs.vec;
    }
}

impl AddAssign<&Vector> for Point
{
    #[inline]
    fn add_assign(&mut self, rhs: &Vector)
    {
        AddAssign::add_assign(self, *rhs);
    }
}

impl AddAssign<Vector> for &mut Point
{
    #[inline]
    fn add_assign(&mut self, rhs: Vector)
    {
        AddAssign::add_assign(*self, rhs);
    }
}

impl AddAssign<&Vector> for &mut Point
{
    #[inline]
    fn add_assign(&mut self, rhs: &Vector)
    {
        AddAssign::add_assign(*self, *rhs);
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn constructors()
    {
        let origin = Point::origin();

        assert_eq!(origin.vec[0], 0.0);
        assert_eq!(origin.vec[1], 0.0);
        assert_eq!(origin.vec[2], 0.0);

        let point = Point::new(1.0, 2.0, 3.0);

        assert_eq!(point.vec[0], 1.0);
        assert_eq!(point.vec[1], 2.0);
        assert_eq!(point.vec[2], 3.0);
    }

    #[test]
    fn components_accessible_by_name()
    {
        let point = Point::new(1.0, 2.0, 3.0);

        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);

        let mut point = Point::origin();
        point.x = 1.0;
        point.y = 2.0;
        point.z = 3.0;

        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);
    }

    #[test]
    fn components_accessible_by_index()
    {
        let point = Point::new(1.0, 2.0, 3.0);

        assert_eq!(point[0], 1.0);
        assert_eq!(point[1], 2.0);
        assert_eq!(point[2], 3.0);

        let mut point = Point::origin();
        point[0] = 1.0;
        point[1] = 2.0;
        point[2] = 3.0;

        assert_eq!(point[0], 1.0);
        assert_eq!(point[1], 2.0);
        assert_eq!(point[2], 3.0);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_access()
    {
        let point = Point::origin();
        let _ = point[3];
    }

    #[test]
    fn sub()
    {
        let a = Point::new(10.0, 10.0, 10.0);
        let b = Point::new(20.0, 30.0, 40.0);

        let v = b - a;

        assert_eq!(v, Vector::new(10.0, 20.0, 30.0));

        let a = &Point::new(10.0, 10.0, 10.0);
        let b = Point::new(20.0, 30.0, 40.0);

        let v = b - a;

        assert_eq!(v, Vector::new(10.0, 20.0, 30.0));

        let a = Point::new(10.0, 10.0, 10.0);
        let b = &Point::new(20.0, 30.0, 40.0);

        let v = b - a;

        assert_eq!(v, Vector::new(10.0, 20.0, 30.0));

        let a = &Point::new(10.0, 10.0, 10.0);
        let b = &Point::new(20.0, 30.0, 40.0);

        let v = b - a;

        assert_eq!(v, Vector::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn add()
    {
        let a = Point::new(10.0, 10.0, 10.0);
        let v = Vector::new(20.0, 30.0, 40.0);

        let b = a + v;

        assert_eq!(b, Point::new(30.0, 40.0, 50.0));

        let a = Point::new(10.0, 10.0, 10.0);
        let v = &Vector::new(20.0, 30.0, 40.0);

        let b = a + v;

        assert_eq!(b, Point::new(30.0, 40.0, 50.0));

        let a = &Point::new(10.0, 10.0, 10.0);
        let v = Vector::new(20.0, 30.0, 40.0);

        let b = a + v;

        assert_eq!(b, Point::new(30.0, 40.0, 50.0));

        let a = &Point::new(10.0, 10.0, 10.0);
        let v = &Vector::new(20.0, 30.0, 40.0);

        let b = a + v;

        assert_eq!(b, Point::new(30.0, 40.0, 50.0));
    }

    #[test]
    fn add_assign()
    {
        let mut a = Point::new(10.0, 10.0, 10.0);
        let v = Vector::new(20.0, 30.0, 40.0);

        a += v;

        assert_eq!(a, Point::new(30.0, 40.0, 50.0));

        let mut a = Point::new(10.0, 10.0, 10.0);
        let v = &Vector::new(20.0, 30.0, 40.0);

        a += v;

        assert_eq!(a, Point::new(30.0, 40.0, 50.0));

        let mut a = &mut Point::new(10.0, 10.0, 10.0);
        let v = Vector::new(20.0, 30.0, 40.0);

        a += v;

        assert_eq!(*a, Point::new(30.0, 40.0, 50.0));

        let mut a = &mut Point::new(10.0, 10.0, 10.0);
        let v = &Vector::new(20.0, 30.0, 40.0);

        a += v;

        assert_eq!(*a, Point::new(30.0, 40.0, 50.0));
    }
}
