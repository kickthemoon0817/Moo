use moo::core::state::PhaseSpace;
use moo::core::solve::{Integrator, VelocityVerlet};
use moo::laws::registry::LawRegistry;
use moo::laws::classical::Spring;
use moo::core::math::ad::Dual;

#[test]
fn test_harmonic_oscillator_conservation() {
    // 1. Setup Phase Space (1 particle, 3 DOF)
    let mut state = PhaseSpace::new(3); // x, y, z
    state.mass[0] = 1.0;
    state.mass[1] = 1.0;
    state.mass[2] = 1.0;

    // Initial Condition: Displaced by 1.0 unit in X
    state.q[0] = 1.0; 
    // Anchor point (implicit, or we can use 2 particles. Let's use 2 particles for clarity)
    state.resize(6); // 2 particles
    state.mass[0] = 1.0; // P1 mass x
    state.mass[1] = 1.0; // P1 mass y
    state.mass[2] = 1.0; // P1 mass z
    state.mass[3] = 1000.0; // P2 (Anchor) heavy mass
    state.mass[4] = 1000.0;
    state.mass[5] = 1000.0;

    state.q[0] = 1.0; // P1 at x=1
    state.q[3] = 0.0; // P2 at x=0 (Anchor)

    // 2. Setup Laws
    let mut registry = LawRegistry::new();
    let k = 10.0;
    let rest_length = 0.0;
    // Spring between P1 (idx 0 -> 0,1,2) and P2 (idx 1 -> 3,4,5)
    registry.add(Spring::new(k, rest_length, 0, 1));

    // 3. Define Energy Calculation Helper
    let calc_energy = |s: &PhaseSpace, laws: &LawRegistry| -> f64 {
        // Kinetic T = 0.5 * m * v^2
        let mut kinetic = 0.0;
        for i in 0..s.dof {
            kinetic += 0.5 * s.mass[i] * s.v[i] * s.v[i];
        }

        // Potential V
        let q_dual: Vec<Dual> = s.q.iter().map(|&x| Dual::constant(x)).collect();
        let potential = laws.potential(&q_dual, &s.mass).val;

        kinetic + potential
    };

    let initial_energy = calc_energy(&state, &registry);
    println!("Initial Energy: {:.6}", initial_energy);

    // 4. Run Simulation
    let mut solver = VelocityVerlet;
    let dt = 0.01;
    let steps = 1000;

    for _ in 0..steps {
        solver.step(&mut state, &registry, &[], dt);
    }

    let final_energy = calc_energy(&state, &registry);
    println!("Final Energy: {:.6}", final_energy);

    // 5. Verification
    // Symplectic integrators oscillate energy but keep it bounded.
    // Error should be small.
    let error = (final_energy - initial_energy).abs();
    println!("Energy Drift: {:.6}", error);

    assert!(error < 1e-2, "Energy drift too high for Symplectic Euler!");
}
