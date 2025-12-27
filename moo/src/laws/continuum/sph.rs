use crate::core::math::ad::Dual;
use crate::laws::registry::Law;
use std::f64::consts::PI;

/// SPH Fluid Law (Lagrangian Formulation)
///
/// We derive pressure forces from an internal potential energy:
/// V = Sum (m_i * u(rho_i))
///
/// Where u(rho) is the internal energy per unit mass.
/// For Tait EOS (Weakly Compressible):
/// P = B * ((rho / rho0)^gamma - 1)
/// u(rho) = (B / rho0) * ( (rho/rho0)^(gamma-1) / (gamma-1) + 1/rho ) ... actually simpler derivation exists.
///
/// Correct Potential Density relationship: P = rho^2 * du/drho
///
/// To simplify for this prototype:
/// We implement a simple "Penalty" Potential for density deviation.
/// V = 0.5 * k * (rho - rho0)^2 / rho0 (Stiffness based)
/// This leads to linear pressure (approx).
///
/// Or proper Tait Potential. Let's try simple stiffness first.
pub struct SPH {
    pub h: f64,    // Smoothing radius
    pub rho0: f64, // Rest density
    pub k: f64,    // Stiffness (Bulk modulus related)
    pub poly6_coeff: f64,
}

impl SPH {
    pub fn new(h: f64, rho0: f64, k: f64) -> Self {
        // Poly6 Kernel constant: 315 / (64 * pi * h^9)
        let poly6_coeff = 315.0 / (64.0 * PI * h.powi(9));
        Self {
            h,
            rho0,
            k,
            poly6_coeff,
        }
    }
}

impl Law for SPH {
    fn potential(&self, q: &[Dual], mass: &[f64]) -> Dual {
        let n = q.len() / 3;
        let mass_stride = if mass.len() == q.len() { 3 } else { 1 };
        let h_sq = self.h * self.h;

        let mut total_potential = Dual::constant(0.0);

        // 1. Calculate Density field (rho) per particle
        // Note: In AD, density is a Dual number dependent on positions q.
        let mut densities = vec![Dual::constant(0.0); n];

        for (i, rho) in densities.iter_mut().enumerate().take(n) {
            let idx_i = i * 3;
            // Self-density contribution (r=0 -> W(0)=315/(64*pi*h^9)*h^6 = 315/(64*pi*h^3))
            // W(0) = 315 / 64pi * h^9 * (h^2)^3 = 315/64pi*h^3
            // Code below handles r=0 naturally if we iterate j including i.

            for j in 0..n {
                let idx_j = j * 3;

                let dx = q[idx_i] - q[idx_j];
                let dy = q[idx_i + 1] - q[idx_j + 1];
                let dz = q[idx_i + 2] - q[idx_j + 2];

                let dist_sq = dx * dx + dy * dy + dz * dz;

                // Poly6 Kernel
                // W(r, h) = coeff * (h^2 - r^2)^3   if 0 <= r <= h
                //         = 0                       otherwise
                // We use a smooth conditional for AD if needed, but strict cutoff is fine for now.

                // Note: branching 'if' with Duals is tricky if we are precisely at h.
                // But generally safe.
                if dist_sq.val < h_sq {
                    let term = Dual::constant(h_sq) - dist_sq;
                    let w = Dual::constant(self.poly6_coeff) * term * term * term;
                    *rho = *rho + Dual::constant(mass[j * mass_stride]) * w;
                }
            }
        }

        // 2. Compute Potential Energy based on Density
        // V = Sum ( (P / rho^2) ) ... no, derived from EOS.
        // Let's use simple Stiffness Potential:
        // V = sum( (rho_i - rho0)^2 ) * Stiffness_volume
        // Actually, potential energy density e = 0.5 * k * (rho - rho0)^2 / rho0
        // Total V = Integral e dV ~ Sum ( e * (m/rho) ) = Sum ( 0.5 * k * (rho-rho0)^2 / (rho * rho0) * m )
        // Using Volume_i = m_i / rho_i is standard.

        for (i, rho) in densities.iter().enumerate().take(n) {
            let m = mass[i * mass_stride];

            // Avoid division by zero
            let vol = if rho.val > 1e-6 {
                Dual::constant(m) / *rho
            } else {
                Dual::constant(0.0)
            };

            let delta = *rho - Dual::constant(self.rho0);
            // Elastic potential energy
            let u = Dual::constant(0.5 * self.k) * delta * delta; // Energy density

            total_potential = total_potential + u * vol;
        }

        total_potential
    }
}
