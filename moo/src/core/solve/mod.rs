use crate::core::math::ad::Dual;
use crate::core::state::PhaseSpace;
use crate::laws::registry::LawRegistry;

pub mod constraints;
use constraints::Constraint;

pub trait Integrator {
    fn step(
        &mut self,
        state: &mut PhaseSpace,
        laws: &LawRegistry,
        constraints: &[Box<dyn Constraint>],
        dt: f64,
    );
}

pub struct SymplecticEuler;

impl Integrator for SymplecticEuler {
    fn step(
        &mut self,
        state: &mut PhaseSpace,
        laws: &LawRegistry,
        constraints: &[Box<dyn Constraint>],
        dt: f64,
    ) {
        let n = state.dof;
        let mut forces = vec![0.0; n];

        // 1. Compute Gradients (Forces) F = -dV/dq
        let mut q_dual: Vec<Dual> = state.q.iter().map(|&x| Dual::constant(x)).collect();

        for i in 0..n {
            q_dual[i].der = 1.0;
            let potential = laws.potential(&q_dual, &state.mass);
            forces[i] = -potential.der;
            q_dual[i].der = 0.0;
        }

        // 2. Symplectic Euler Step
        for (i, f) in forces.iter().enumerate().take(n) {
            let acceleration = f / state.mass[i];
            state.v[i] += acceleration * dt;
            state.q[i] += state.v[i] * dt;
        }

        // 3. Constraints
        for c in constraints {
            c.project(state);
        }

        state.t += dt;
    }
}

pub struct VelocityVerlet;

impl Integrator for VelocityVerlet {
    fn step(
        &mut self,
        state: &mut PhaseSpace,
        laws: &LawRegistry,
        constraints: &[Box<dyn Constraint>],
        dt: f64,
    ) {
        let n = state.dof;

        let mut forces = vec![0.0; n];
        let mut inputs: Vec<Dual> = state.q.iter().map(|&x| Dual::constant(x)).collect();

        // Compute Forces F(t)
        for (i, force) in forces.iter_mut().enumerate().take(n) {
            inputs[i].der = 1.0;
            let potential = laws.potential(&inputs, &state.mass);
            *force = -potential.der;
            inputs[i].der = 0.0;
        }

        // 1. Half Kick v += 0.5 * a * dt
        for (i, v) in state.v.iter_mut().enumerate().take(n) {
            let a = forces[i] / state.mass[i];
            *v += 0.5 * a * dt;
        }

        // 2. Drift x += v * dt
        for (i, q) in state.q.iter_mut().enumerate().take(n) {
            *q += state.v[i] * dt;
        }

        // --- Constraints Projection ---
        for c in constraints {
            c.project(state);
        }

        // 3. Compute Forces F(t+dt) with new positions
        for (i, input) in inputs.iter_mut().enumerate().take(n) {
            input.val = state.q[i]; // Update positions in dual vector
        }

        for (i, force) in forces.iter_mut().enumerate().take(n) {
            inputs[i].der = 1.0;
            let potential = laws.potential(&inputs, &state.mass);
            *force = -potential.der;
            inputs[i].der = 0.0;
        }

        // 4. Half Kick v += 0.5 * new_a * dt
        for (i, v) in state.v.iter_mut().enumerate().take(n) {
            let a = forces[i] / state.mass[i];
            *v += 0.5 * a * dt;
        }

        // --- Rigid Body Rotation Step (Splitting Method) ---
        let rb_count = state.rot.len();
        if rb_count > 0 {
            use crate::core::geometry::{Manifold, SO3};

            for (omega, (inertia, rot)) in state
                .ang_v
                .iter_mut()
                .zip(state.inertia.iter().zip(state.rot.iter_mut()))
            {
                let iw = *omega * *inertia;
                let w_x_iw = omega.cross(iw);
                let d_omega = -w_x_iw / *inertia;

                *omega += d_omega * dt;

                let delta_rot = *omega * dt;
                *rot = SO3::retract(*rot, delta_rot);
            }
        }

        state.t += dt;
    }
}
