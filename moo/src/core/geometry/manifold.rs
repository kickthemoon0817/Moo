use std::ops::{Add, Sub, Mul, Neg};

/// A smooth manifold where local motion is defined by a tangent space.
/// This allows us to separate the "State" (Point) from the "Change" (Tangent).
pub trait Manifold {
    /// The point on the manifold (e.g., a Rotation quaternion, a Vector position).
    type Point: Clone + Copy + std::fmt::Debug;
    
    /// The vector in the tangent space (e.g., Angular velocity, Linear velocity).
    type Tangent: Clone + Copy + std::fmt::Debug + 
                  Add<Output = Self::Tangent> + 
                  Sub<Output = Self::Tangent> + 
                  Neg<Output = Self::Tangent> + 
                  Mul<f64, Output = Self::Tangent>;

    /// The dimension of the manifold.
    fn dim() -> usize;

    /// Moves a point `p` along the tangent vector `v`.
    /// For Vector Spaces, this is `p + v`.
    /// For Lie Groups, this is `p * exp(v)`.
    fn retract(p: Self::Point, v: Self::Tangent) -> Self::Point;

    /// Finds the tangent vector `v` such that `retract(p, v) \approx q`.
    /// Ideally `local(p, retract(p, v)) == v`.
    /// For Vector Spaces, this is `q - p`.
    /// For Lie Groups, this is `log(p^{-1} * q)`.
    fn local(p: Self::Point, q: Self::Point) -> Self::Tangent;
}
