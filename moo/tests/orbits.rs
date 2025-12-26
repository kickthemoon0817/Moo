use moo::core::state::PhaseSpace;
use moo::core::solve::{Integrator, VelocityVerlet};
use moo::laws::registry::LawRegistry;
use moo::laws::classical::Gravity;
use glam::DVec3;

#[test]
fn test_circular_orbit_stability() {
    let mut state = PhaseSpace::new(6); // 2 Bodies, 3 DOF each

    let m1 = 1000.0_f64; // Sun
    let m2 = 10.0_f64;   // Earth
    let dist = 100.0_f64;
    let g = 1.0_f64;
    
    // Calculate required velocity for circular orbit
    // F_g = G * m1 * m2 / r^2
    // F_c = m2 * v^2 / r
    // G * m1 / r^2 = v^2 / r  => v = sqrt(G * m1 / r) (Assuming m1 >> m2 or using reduced mass)
    // Precise: v_rel = sqrt(G(m1+m2)/r)
    // We put Center of Mass at origin.
    // r1 = -m2 / (m1+m2) * r
    // r2 = m1 / (m1+m2) * r
    // v1 = m2 / (m1+m2) * v_rel
    // v2 = -m1 / (m1+m2) * v_rel
    
    let mu = g * (m1 + m2);
    let v_rel_mag = (mu / dist).sqrt();
    
    let r_vec = DVec3::new(dist, 0.0, 0.0);
    let v_vec = DVec3::new(0.0, v_rel_mag, 0.0); // Perpendicular velocity

    // Body 1 (Heavy)
    state.mass[0] = m1;
    // Body 2 (Light)
    state.mass[1] = m2;

    // Set Positions (Relative)
    // Let's keep Body 1 at origin for simplicity (approximate if m1 >> m2)
    // But to be exact without fixing Body 1, we sets velocities such that momentum is zero.
    
    let frac2 = m2 / (m1 + m2);
    let frac1 = m1 / (m1 + m2);
    
    // Positions
    let q1 = -r_vec * frac2;
    let q2 = r_vec * frac1;
    
    state.q[0] = q1.x; state.q[1] = q1.y; state.q[2] = q1.z;
    state.q[3] = q2.x; state.q[4] = q2.y; state.q[5] = q2.z;

    // Velocities
    let v1 = -v_vec * frac2;
    let v2 = v_vec * frac1;

    state.v[0] = v1.x; state.v[1] = v1.y; state.v[2] = v1.z;
    state.v[3] = v2.x; state.v[4] = v2.y; state.v[5] = v2.z;

    let mut registry = LawRegistry::new();
    registry.add(Gravity::new(g));

    let mut solver = VelocityVerlet;
    let dt = 0.001;
    let steps = 10000;
    
    // Run simulation
    let p1_0 = DVec3::from_slice(&state.q[0..3]);
    let p2_0 = DVec3::from_slice(&state.q[3..6]);
    println!("Initial Pos: P1={:?}, P2={:?}", p1_0, p2_0);
    println!("Initial Vel: P1={:?}, P2={:?}", DVec3::from_slice(&state.v[0..3]), DVec3::from_slice(&state.v[3..6]));
    
    // Step 1
    solver.step(&mut state, &registry, &[], dt);
    
    let v1_1 = DVec3::from_slice(&state.v[0..3]);
    let v2_1 = DVec3::from_slice(&state.v[3..6]);
    let a1_est = (v1_1 - DVec3::from_slice(&[v1.x, v1.y, v1.z])) / dt;
    let a2_est = (v2_1 - DVec3::from_slice(&[v2.x, v2.y, v2.z])) / dt;
    
    println!("After Step 1:");
    println!("Est Accel P1: {:?} (Expected mag: 0.001)", a1_est);
    println!("Est Accel P2: {:?} (Expected mag: 0.1)", a2_est);
    
    for _ in 1..steps {
        solver.step(&mut state, &registry, &[], dt);
    }
    
    // Check if distance is still ~100.0
    let p1 = DVec3::from_slice(&state.q[0..3]);
    let p2 = DVec3::from_slice(&state.q[3..6]);
    let final_dist = (p1 - p2).length();
    
    println!("Initial Dist: {}, Final Dist: {}", dist, final_dist);
    assert!((final_dist - dist).abs() < 1e-1, "Orbit should remain stable (circular)");
}
