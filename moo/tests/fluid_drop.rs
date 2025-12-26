use moo::core::state::PhaseSpace;
use moo::laws::registry::Law;
use moo::laws::continuum::SPH;
use moo::core::math::ad::Dual;

#[test]
fn test_sph_pressure_repulsion() {
    // 1. Setup
    let mut state = PhaseSpace::new(6); // 2 particles
    
    // Place them very close (distance 0.5)
    // Smoothing length h = 1.0
    // Rest density rho0 = 1.0
    state.q[0] = 0.0; state.q[1] = 0.0; state.q[2] = 0.0;
    state.q[3] = 0.5; state.q[4] = 0.0; state.q[5] = 0.0;
    
    // Mass 1.0
    state.set_particle_mass(0, 1.0);
    state.set_particle_mass(1, 1.0);

    // 2. Define SPH Law
    let h = 1.0;
    let rho0 = 1.0;
    let k = 100.0; // High stiffness
    let sph = SPH::new(h, rho0, k);

    // 3. Compute Potential manually
    let q_dual: Vec<Dual> = state.q.iter().map(|&x| Dual::constant(x)).collect();
    
    // To get force on P2 (x-direction, index 3)
    let mut inputs = q_dual.clone();
    inputs[3].der = 1.0; // differentiate w.r.t x2
    
    let potential = sph.potential(&inputs, &state.mass);
    let force_x2 = -potential.der;
    
    println!("Potential Energy: {}", potential.val);
    println!("Force on P2 (x): {}", force_x2);
    
    // 4. Assertions
    // Particles are closer (0.5) than h (1.0). Density should be high.
    // Pressure should push them apart.
    // P2 is at 0.5, P1 at 0.0. P2 should be pushed +x.
    // Force > 0.
    
    assert!(force_x2 > 0.0, "SPH Pressure should be repulsive when compressed! F={}", force_x2);
}
