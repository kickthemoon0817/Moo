use super::manifold::Manifold;
use glam::DVec3;

#[derive(Debug, Clone, Copy, Default)]
pub struct Euclidean3;

impl Manifold for Euclidean3 {
    type Point = DVec3;
    type Tangent = DVec3;

    fn dim() -> usize { 3 }

    fn retract(p: Self::Point, v: Self::Tangent) -> Self::Point {
        p + v
    }

    fn local(p: Self::Point, q: Self::Point) -> Self::Tangent {
        q - p
    }
}
