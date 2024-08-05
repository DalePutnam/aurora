use std::ops::Deref;
use std::ops::Neg;

use super::Vector;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Normal
{
    pub(super) vec: Vector,
}

impl Normal
{
    pub fn from_vector(vector: &Vector) -> Self
    {
        Normal {
            vec: vector.normalize(),
        }
    }

    pub fn x_axis() -> Self
    {
        Normal {
            vec: Vector::new(1.0, 0.0, 0.0),
        }
    }

    pub fn y_axis() -> Self
    {
        Normal {
            vec: Vector::new(0.0, 1.0, 0.0),
        }
    }

    pub fn z_axis() -> Self
    {
        Normal {
            vec: Vector::new(0.0, 0.0, 1.0),
        }
    }
}

impl Deref for Normal
{
    type Target = Vector;

    #[inline]
    fn deref(&self) -> &Self::Target
    {
        &self.vec
    }
}

impl Neg for Normal
{
    type Output = Normal;

    fn neg(self) -> Normal
    {
        Normal { vec: -self.vec }
    }
}

impl Neg for &Normal
{
    type Output = Normal;

    fn neg(self) -> Normal
    {
        Normal { vec: -self.vec }
    }
}

#[cfg(test)]
mod tests
{
    use super::Vector;
    use super::*;

    #[test]
    fn from_vector()
    {
        let vector = Vector::new(1.0, 2.0, 2.0);
        let normal = Normal::from_vector(&vector);

        assert_eq!(normal[0], 1.0 / 3.0);
        assert_eq!(normal[1], 2.0 / 3.0);
        assert_eq!(normal[2], 2.0 / 3.0);
    }

    #[test]
    fn call_vector_methods()
    {
        // Don't care about results, just needs to compile

        let normal = Normal::from_vector(&Vector::new(1.0, 2.0, 3.0));

        normal.magnitude();
        normal.cross(&normal);
        normal.dot(&normal);

        normal.x;
        normal.y;
        normal.z;
    }
}
