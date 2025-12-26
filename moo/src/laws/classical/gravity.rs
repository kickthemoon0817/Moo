use crate::core::math::ad::Dual;
use crate::laws::registry::Law;

/// Newtonian Gravity: V = -G * m1 * m2 / r
pub struct Gravity {
    pub g: f64,
}

impl Gravity {
    pub fn new(g: f64) -> Self {
        Self { g }
    }
}

impl Law for Gravity {
    fn potential(&self, q: &[Dual], mass: &[f64]) -> Dual {
        let mut total_potential = Dual::constant(0.0);
        let n_particles = q.len() / 3;
        
        // Ensure mass definition is consistent
        // If mass.len() == q.len() (Per DOF mass), use stride 3.
        // If mass.len() == n_particles (Per particle mass), use stride 1.
        let mass_stride = if mass.len() == q.len() { 3 } else { 1 };

        if q.len() % 3 != 0 {
             return Dual::constant(0.0);
        }

        for i in 0..n_particles {
            for j in (i + 1)..n_particles {
                let idx_i = i * 3;
                let idx_j = j * 3;

                let dx = q[idx_i] - q[idx_j];
                let dy = q[idx_i+1] - q[idx_j+1];
                let dz = q[idx_i+2] - q[idx_j+2]; 

                let dist_sq = dx * dx + dy * dy + dz * dz; 
                let dist = Dual::new(dist_sq.val.sqrt(), 0.5 * dist_sq.der / dist_sq.val.sqrt()); 

                // V = -G * m1 * m2 / r
                let m1m2 = mass[i * mass_stride] * mass[j * mass_stride];
                let term = dist.inv() * Dual::constant(-self.g * m1m2);
                
                total_potential = total_potential + term;
            }
        }

        total_potential
    }
}

impl Dual {
    fn inv(self) -> Self {
        // 1/x -> -1/x^2
        Self::new(1.0 / self.val, -self.der / (self.val * self.val))
    }
}
