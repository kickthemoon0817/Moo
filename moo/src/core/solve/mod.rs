use crate::core::state::PhaseSpace;
use crate::core::math::ad::Dual;
use crate::laws::registry::LawRegistry;

pub trait Integrator {
    fn step(&mut self, state: &mut PhaseSpace, laws: &LawRegistry, dt: f64);
}

pub struct SymplecticEuler;

impl Integrator for SymplecticEuler {
    fn step(&mut self, state: &mut PhaseSpace, laws: &LawRegistry, dt: f64) {
        let n = state.dof;
        let mut forces = vec![0.0; n];

        // 1. Compute Gradients (Forces) F = -\nabla V(q)
        // Optimization Note: This naive Forward-Mode AD pass is O(N * Cost(V)).
        // For pairwise potentials, this is O(N^3). 
        // In production, laws should provide analytical gradients or use Reverse-Mode AD.
        // For this baseline, we prioritize "Correctness" of the interface.
        
        // We reuse a vector for q_dual to avoid allocation
        let mut q_dual: Vec<Dual> = state.q.iter().map(|&x| Dual::constant(x)).collect();

        for i in 0..n {
            // Seed the derivative for q_i
            q_dual[i].der = 1.0;
            
            let potential = laws.potential(&q_dual, &state.mass);
            forces[i] = -potential.der; // F = -dV/dq

            // Reset seed
            q_dual[i].der = 0.0;
        }

        // 2. Symplectic Euler Step applies Conservation
        // v_{t+1} = v_t + F/m * dt
        // q_{t+1} = q_t + v_{t+1} * dt

        for i in 0..n {
            let acceleration = forces[i] / state.mass[i];
            state.v[i] += acceleration * dt;
            state.q[i] += state.v[i] * dt;
        }

        state.t += dt;
    }
}

pub struct VelocityVerlet;

impl Integrator for VelocityVerlet {
    fn step(&mut self, state: &mut PhaseSpace, laws: &LawRegistry, dt: f64) {
        let n = state.dof;
        
        // Velocity Verlet:
        // 1. v(t + 0.5dt) = v(t) + 0.5 * a(t) * dt
        // 2. x(t + dt) = x(t) + v(t + 0.5dt) * dt
        // 3. Compute a(t + dt) from x(t + dt)
        // 4. v(t + dt) = v(t + 0.5dt) + 0.5 * a(t + dt) * dt

        // We need forces at t. Ideally we cache them, but for now we recompute or assume passed in.
        // For this simple implementation, we compute a(t) first.
        
        let mut forces = vec![0.0; n];
        let mut inputs: Vec<Dual> = state.q.iter().map(|&x| Dual::constant(x)).collect();

        // Compute Forces F(t)
        for i in 0..n {
            inputs[i].der = 1.0;
            let potential = laws.potential(&inputs, &state.mass);
            forces[i] = -potential.der;
            inputs[i].der = 0.0;
        }

        // 1. Half Kick v += 0.5 * a * dt
        for i in 0..n {
            let a = forces[i] / state.mass[i];
            state.v[i] += 0.5 * a * dt;
        }

        // 2. Drift x += v * dt
        for i in 0..n {
            state.q[i] += state.v[i] * dt;
        }

        // 3. Compute Forces F(t+dt) with new positions
        for i in 0..n {
            inputs[i].val = state.q[i]; // Update positions in dual vector
        }
        
        for i in 0..n {
            inputs[i].der = 1.0;
            let potential = laws.potential(&inputs, &state.mass);
            forces[i] = -potential.der;
            inputs[i].der = 0.0;
        }

        // 4. Half Kick v += 0.5 * new_a * dt
        for i in 0..n {
            let a = forces[i] / state.mass[i];
            state.v[i] += 0.5 * a * dt;
        }

        // --- Rigid Body Rotation Step (Splitting Method) ---
        // For Prototype 1, we assume Torque-Free motion (Angular Momentum conserved)
        // Euler's Equations: I_body * d\Omega/dt + \Omega x (I_body * \Omega) = 0
        // d\Omega/dt = -I_body^{-1} * (\Omega x (I_body * \Omega))
        
        let rb_count = state.rot.len();
        if rb_count > 0 {
            use crate::core::geometry::{Manifold, SO3};
            
            for i in 0..rb_count {
                let omega = state.ang_v[i];
                let inertia = state.inertia[i]; // Principal moments
                
                // Explicit Euler for Omega (sufficient for short dt, improvement: RK4 for Rotation)
                // Torque T = 0
                // dOmega = -I^-1 * (w x Iw)
                let iw = omega * inertia; // component-wise mult since inertia is diagonal
                let w_x_iw = omega.cross(iw);
                let d_omega = -w_x_iw / inertia; // component-wise div
                
                // Update Omega
                state.ang_v[i] += d_omega * dt;
                
                // Update Orientation R_{n+1} = R_n * exp(Omega * dt)
                // We use Body Frame update: q_new = q * exp(omega * dt)
                let delta_rot = state.ang_v[i] * dt;
                state.rot[i] = SO3::retract(state.rot[i], delta_rot);
            }
        }
        
        state.t += dt;
    }
}
