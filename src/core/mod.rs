pub use self::primitives::NonhierBox;
pub use self::primitives::NonhierSphere;
pub use self::material::Material;
pub use self::ray::Ray;
pub use self::ray::Hit;

pub mod traits;
pub mod math;
pub mod primitives;
pub mod material;
pub mod ray;
pub mod object;