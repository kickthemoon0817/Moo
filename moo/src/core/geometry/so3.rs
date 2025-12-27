use crate::core::geometry::manifold::Manifold;
use glam::{DQuat, DVec3};

/// The Special Orthogonal Group SO(3) representing 3D rotations.
/// We use Unit Quaternions for implementation.
#[derive(Debug, Clone, Copy, Default)]
pub struct SO3;

impl Manifold for SO3 {
    type Point = DQuat;
    /// Tangent vector in the Lie Algebra so(3).
    /// Represents angular velocity \omega.
    type Tangent = DVec3;

    fn dim() -> usize {
        3
    }

    /// Retraction map: exponential map on SO(3).
    /// Updates rotation `q` by angular velocity `v` over time 1.
    /// q_{new} = q * exp(v)
    fn retract(q: Self::Point, v: Self::Tangent) -> Self::Point {
        // Formally on Lie Groups: exp: bundle -> manifold
        // For Quaternions: q_new = q + 0.5 * w * q * dt (linearized)
        // Or exact: q_new = q * [cos(|w|/2), sin(|w|/2) * w/|w|]

        // glam's `DQuat::from_scaled_axis(v)` computes exp(v/2) efficiently.
        // Note: The factor of 1/2 usually comes from the definition of the algebra.
        // If v is angular velocity vector w, the update is typically exp(w * dt / 2) in quaternion space?
        // Let's assume v is the "scaled axis" angle vector.

        let delta = DQuat::from_scaled_axis(v);
        (q * delta).normalize()
    }

    /// Logarithmic map (Inverse Retraction).
    /// Finds v such that retract(p, v) = q (approx).
    /// v = log(p^{-1} * q)
    fn local(p: Self::Point, q: Self::Point) -> Self::Tangent {
        let delta = p.inverse() * q;
        delta.to_scaled_axis()
    }
}
