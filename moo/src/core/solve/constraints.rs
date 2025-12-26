use crate::core::state::PhaseSpace;

/// A geometric constraint that enforces non-penetration or joints.
pub trait Constraint {
    /// Projects the state to satisfy the constraint.
    /// Modifies position (q) and velocity (v).
    fn project(&self, state: &mut PhaseSpace);
}

pub struct FloorConstraint {
    pub y_level: f64,
    pub restitution: f64,
}

impl FloorConstraint {
    pub fn new(y_level: f64, restitution: f64) -> Self {
        Self { y_level, restitution }
    }
}

impl Constraint for FloorConstraint {
    fn project(&self, state: &mut PhaseSpace) {
        let n = state.dof / 3;
        for i in 0..n {
            let idx = i * 3;
            let y = state.q[idx + 1];
            
            // Check penetration
            if y < self.y_level {
                // Positional Projection
                state.q[idx + 1] = self.y_level;
                
                // Velocity Reflection (Impulse)
                let vy = state.v[idx + 1];
                if vy < 0.0 {
                    state.v[idx + 1] = -vy * self.restitution;
                    
                    // Friction (Simple)
                    let friction = 0.9;
                    state.v[idx] *= friction;
                    state.v[idx+2] *= friction;
                }
            }
        }
    }
}

pub struct SphereConstraint {
    pub restitution: f64,
    /// Minimum separation used to avoid division by zero.
    pub min_separation: f64,
}

impl SphereConstraint {
    pub fn new(restitution: f64) -> Self {
        Self {
            restitution,
            min_separation: DEFAULT_MIN_SEPARATION,
        }
    }

    pub fn with_min_separation(restitution: f64, min_separation: f64) -> Self {
        Self {
            restitution,
            min_separation: min_separation.abs(),
        }
    }
}

const DEFAULT_MIN_SEPARATION: f64 = 1e-6;

impl Constraint for SphereConstraint {
    fn project(&self, state: &mut PhaseSpace) {
        let n = state.dof / 3;
        let min_sep = self.min_separation.max(DEFAULT_MIN_SEPARATION);
        let min_sep_sq = min_sep * min_sep;
        for i in 0..n {
            for j in (i + 1)..n {
                let idx_i = i * 3;
                let idx_j = j * 3;

                let p1 = glam::DVec3::from_slice(&state.q[idx_i..idx_i+3]);
                let p2 = glam::DVec3::from_slice(&state.q[idx_j..idx_j+3]);
                
                let diff = p1 - p2;
                let dist_sq = diff.length_squared();
                let r_sum = state.radius[i] + state.radius[j];

                if dist_sq < r_sum * r_sum {
                    let v1 = glam::DVec3::from_slice(&state.v[idx_i..idx_i+3]);
                    let v2 = glam::DVec3::from_slice(&state.v[idx_j..idx_j+3]);
                    let rel_vel = v1 - v2;

                    let (normal, dist) = if dist_sq < min_sep_sq {
                        // Fallback normal to avoid NaNs when particles fully overlap.
                        let mut fallback = rel_vel.normalize_or_zero();
                        if fallback.length_squared() == 0.0 {
                            fallback = glam::DVec3::X;
                        }
                        (fallback, min_sep)
                    } else {
                        let dist = dist_sq.sqrt();
                        (diff / dist, dist)
                    };

                    let overlap = r_sum - dist;
                    if overlap <= 0.0 {
                        continue;
                    }

                    // 1. Positional Correction (Projection)
                    // Split overlap based on inverse mass? (Simplification: 0.5 each for now)
                    let correction = normal * (overlap * 0.5);
                    
                    state.q[idx_i] += correction.x;
                    state.q[idx_i+1] += correction.y;
                    state.q[idx_i+2] += correction.z;

                    state.q[idx_j] -= correction.x;
                    state.q[idx_j+1] -= correction.y;
                    state.q[idx_j+2] -= correction.z;

                    // 2. Velocity Response
                    let vel_along_normal = rel_vel.dot(normal);

                    if vel_along_normal < 0.0 {
                        let j_impulse = -(1.0 + self.restitution) * vel_along_normal;
                        // Assuming equal mass for impulse distribution simplicity in this constraint
                        // Ideally: j / (1/m1 + 1/m2). 
                        // Let's do it properly if we can access mass.
                        let inv_mass1 = 1.0 / state.mass[i * 3]; // Mass is duplicated per DOF
                        let inv_mass2 = 1.0 / state.mass[j * 3];
                        let impulse_mag = j_impulse / (inv_mass1 + inv_mass2);
                        
                        let impulse = normal * impulse_mag;
                        
                        // Apply Impulse
                        state.v[idx_i] += impulse.x * inv_mass1;
                        state.v[idx_i+1] += impulse.y * inv_mass1;
                        state.v[idx_i+2] += impulse.z * inv_mass1;

                        state.v[idx_j] -= impulse.x * inv_mass2;
                        state.v[idx_j+1] -= impulse.y * inv_mass2;
                        state.v[idx_j+2] -= impulse.z * inv_mass2;
                    }
                }
            }
        }
    }
}
