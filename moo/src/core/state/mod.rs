use crate::core::math::ad::Dual;

/// Represents the State of the system in Phase Space (q, p).
/// 
/// We use a Structure-of-Arrays (SoA) layout. Instead of having a `Vec<Particle>`
/// where each particle has position and velocity, we have one giant `Vec<f64>` for positions
/// and one for momenta.
/// 
/// This is critical for cache coherence when the Integrator iterates over the state.
#[derive(Debug, Clone, Default)]
pub struct PhaseSpace {
    /// Dimension of the configuration space (number of degrees of freedom).
    pub dof: usize,

    /// Generalized Coordinates (q).
    /// Size: `dof`
    pub q: Vec<f64>,

    /// Generalized Velocities (v = \dot{q}) or Momenta (p).
    /// In a Lagrangian formulation, we often track (q, v).
    /// In a Hamiltonian formulation, we track (q, p).
    /// Size: `dof`
    pub v: Vec<f64>,

    /// Mass/Inertia diagonal (simplified). 
    /// Realistically this would be a Mass Matrix, but for SoA particles, a diagonal is often sufficient.
    pub mass: Vec<f64>,

    /// Collision/Geometric Radius.
    /// Size: `dof / 3` (One per particle/body).
    pub radius: Vec<f64>,

    // --- Rigid Body Extensions ---
    /// Orientations (Quaternions).
    /// If empty, system is purely particles.
    /// Indices here might correspond to indices in q/v (if mixed) or be separate.
    /// For Prototype 2, let's assume we can have particles AND rigid bodies.
    /// But simplest is: A rigid body i has center of mass at q[3i..3i+3] and rotation at rot[i].
    pub rot: Vec<glam::DQuat>,
    
    /// Angular Velocities (Body frame).
    pub ang_v: Vec<glam::DVec3>,
    
    /// Inertia Tensor diagonals (Principal moments).
    pub inertia: Vec<glam::DVec3>,

    /// Current time of the state snapshot.
    pub t: f64,
}

impl PhaseSpace {
    pub fn new(dof: usize) -> Self {
        Self {
            dof,
            q: vec![0.0; dof],
            v: vec![0.0; dof],
            mass: vec![1.0; dof],
            radius: vec![1.0; dof / 3],
            rot: Vec::new(),
            ang_v: Vec::new(),
            inertia: Vec::new(),
            t: 0.0,
        }
    }

    /// Resize the state container (e.g., adding particles).
    pub fn resize(&mut self, new_dof: usize) {
        self.dof = new_dof;
        self.q.resize(new_dof, 0.0);
        self.v.resize(new_dof, 0.0);
        self.mass.resize(new_dof, 1.0);
        self.radius.resize(new_dof / 3, 1.0);
    }
    
    /// Resize rigid body storage.
    pub fn resize_rigid(&mut self, count: usize) {
        self.rot.resize(count, glam::DQuat::IDENTITY);
        self.ang_v.resize(count, glam::DVec3::ZERO);
        self.inertia.resize(count, glam::DVec3::ONE);
    }
}

/// A "View" into the state that supports Automatic Differentiation.
/// The Solvers (Integrators) work with this view to compute gradients \nabla L.
pub struct StateView<'a> {
    pub q: &'a [Dual],
    pub v: &'a [Dual],
}
