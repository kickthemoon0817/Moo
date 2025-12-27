use crate::core::math::ad::Dual;

/// A Physical Law that governs the evolution of the system.
///
/// In PhysicLaw, we avoid hardcoding forces. Instead, we define laws via their
/// **Potential Energy** $V(q)$. The engine automatically computes the forces
/// $F = -\nabla V(q)$ using Automatic Differentiation.
///
/// This ensures strict energy conservation (symplecticity) because the forces
/// are guaranteed to be conservative gradients.
pub trait Law {
    /// Computes the total potential energy of the system given the state configuration `q`.
    ///
    /// # Arguments
    /// * `q` - The generalized coordinates in Dual number form (for AD).
    /// * `mass` - The mass constants of the degrees of freedom.
    fn potential(&self, q: &[Dual], mass: &[f64]) -> Dual;
}

/// A registry that aggregates multiple laws.
/// $V_{total} = \sum V_i$
pub struct LawRegistry {
    laws: Vec<Box<dyn Law>>,
}

impl Default for LawRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LawRegistry {
    pub fn new() -> Self {
        Self { laws: Vec::new() }
    }

    pub fn add(&mut self, law: impl Law + 'static) {
        self.laws.push(Box::new(law));
    }

    pub fn potential(&self, q: &[Dual], mass: &[f64]) -> Dual {
        let mut total = Dual::new(0.0, 0.0);
        for law in &self.laws {
            total = total + law.potential(q, mass);
        }
        total
    }
}
