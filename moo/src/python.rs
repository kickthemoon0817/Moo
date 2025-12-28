use pyo3::prelude::*;
// use crate::simulation::Simulation;

// Python Wrapper for Simulation
// Note: Direct access to wgpu/simulation requires careful resource management.
// For Phase 17, we will expose a placeholder class to prove the bindings work.
// Full PyO3 <-> WGPU integration is complex.

#[pyclass]
pub struct MooSim {
    // We can't easily hold the Rust `Simulation` struct here because PyO3 expects Send+Sync or GIL safety.
    // And `Simulation` holds `wgpu` resources which are !Send in some cases (though wgpu resources are mostly Send).
    // For now, we will store basic config.
    n_particles: u32,
}

#[pymethods]
impl MooSim {
    #[new]
    fn new(n_particles: u32) -> Self {
        MooSim { n_particles }
    }

    fn info(&self) -> String {
        format!("Moo Simulation with {} particles", self.n_particles)
    }

    // TODO: Implement actual simulation loop access or connect to gRPC client here.
    // The "Direct Access" mode mentioned in plans requires exposing the Simulation struct
    // effectively, which might need `unsafe` or `Arc<Mutex<Simulation>>`.
}

#[pymodule]
fn moo(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MooSim>()?;
    Ok(())
}
