use crate::core::math::ad::Dual;
use crate::laws::registry::Law;

pub struct Spring {
    pub k: f64,
    pub rest_length: f64,
    pub p1_idx: usize,
    pub p2_idx: usize,
}

impl Spring {
    pub fn new(k: f64, rest_length: f64, p1_idx: usize, p2_idx: usize) -> Self {
        Self {
            k,
            rest_length,
            p1_idx,
            p2_idx,
        }
    }
}

impl Law for Spring {
    fn potential(&self, q: &[Dual], _mass: &[f64]) -> Dual {
        let idx1 = self.p1_idx * 3;
        let idx2 = self.p2_idx * 3;

        if idx1 + 2 >= q.len() || idx2 + 2 >= q.len() {
            return Dual::constant(0.0);
        }

        let dx = q[idx1] - q[idx2];
        let dy = q[idx1 + 1] - q[idx2 + 1];
        let dz = q[idx1 + 2] - q[idx2 + 2];

        let dist_sq = dx * dx + dy * dy + dz * dz;
        // Manual sqrt for Dual
        let dist_val = dist_sq.val.sqrt();
        let dist = if dist_val > 1e-6 {
            Dual::new(dist_val, 0.5 * dist_sq.der / dist_val)
        } else {
            Dual::constant(0.0) // Handle singularity
        };

        // V = 0.5 * k * (r - r0)^2
        let displacement = dist - Dual::constant(self.rest_length);

        displacement * displacement * Dual::constant(0.5 * self.k)
    }
}
