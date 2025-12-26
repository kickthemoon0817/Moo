use moo::core::state::PhaseSpace;
use moo::core::solve::{Integrator, VelocityVerlet};
use moo::laws::registry::LawRegistry;
use glam::DVec3;

#[test]
fn test_rigid_body_energy_conservation() {
    // 1. Setup Phase Space (1 Rigid Body)
    let mut state = PhaseSpace::new(0); // 0 particles
    state.resize_rigid(1);
    
    // 2. Setup Asymmetric Top (I1 < I2 < I3)
    // e.g. a brick: 1 x 2 x 3 dimensions implies I_x proportional to 2^2+3^2=13, etc.
    // Let's just set I diagonal directly.
    let inertia = DVec3::new(1.0, 2.0, 3.0);
    state.inertia[0] = inertia;
    
    // 3. Initial Condition: Spin around intermediate axis (unstable) + slight perturbation
    // If it was exactly around Y (2.0), it would stay there. 
    // Perturbation ensures it tumbles (Dzhanibekov effect).
    state.ang_v[0] = DVec3::new(0.1, 5.0, 0.1); 

    // 4. Registry (Empty, no potentials)
    let registry = LawRegistry::new();

    // 5. Energy Calc Helper
    let calc_rot_energy = |s: &PhaseSpace| -> f64 {
        let w = s.ang_v[0];
        let i = s.inertia[0];
        // T = 0.5 * (Ix*wx^2 + Iy*wy^2 + Iz*wz^2)
        0.5 * w.dot(w * i)
    };

    let initial_energy = calc_rot_energy(&state);
    println!("Initial Rot Energy: {:.6}", initial_energy);

    // 6. Run Simulation
    let mut solver = VelocityVerlet;
    let dt = 0.001; // Smaller dt for rotation stability
    let steps = 5000;

    for _ in 0..steps {
        solver.step(&mut state, &registry, &[], dt);
    }

    let final_energy = calc_rot_energy(&state);
    println!("Final Rot Energy: {:.6}", final_energy);
    
    let error = (final_energy - initial_energy).abs();
    println!("Energy Drift: {:.6}", error);

    // Tolerance check. 
    // Explicit Euler for rotation is O(dt). With dt=0.001 and 5000 steps, drift might be noticeable.
    assert!(error < 1.0, "Rotational energy drift too high! Stability issues?"); 
}
