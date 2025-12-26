use crate::core::state::PhaseSpace;
use crate::laws::registry::LawRegistry;
use crate::core::math::ad::Dual;

/// A synchronous probe that extracts a scalar value from the system state.
pub trait Probe {
    fn name(&self) -> &str;
    fn measure(&self, state: &PhaseSpace, laws: &LawRegistry) -> f64;
}

pub struct EnergyProbe;

impl Probe for EnergyProbe {
    fn name(&self) -> &str {
        "Total Energy"
    }

    fn measure(&self, state: &PhaseSpace, laws: &LawRegistry) -> f64 {
        let n = state.dof;
        
        // 1. Translational Kinetic T = 0.5 * m * v^2
        let mut kinetic = 0.0;
        for i in 0..n {
            kinetic += 0.5 * state.mass[i] * state.v[i] * state.v[i];
        }

        // 2. Rotational Kinetic T_rot = 0.5 * w . (I * w)
        let mut rot_kinetic = 0.0;
        for i in 0..state.rot.len() {
            let w = state.ang_v[i];
            let inertia = state.inertia[i];
            rot_kinetic += 0.5 * w.dot(w * inertia);
        }

        // 3. Potential V
        // We need Dual numbers for the potential function
        // Optimization: We only need value, so derivative seed can be 0.
        let q_dual: Vec<Dual> = state.q.iter().map(|&x| Dual::constant(x)).collect();
        let potential = laws.potential(&q_dual, &state.mass).val;

        kinetic + rot_kinetic + potential
    }
}
