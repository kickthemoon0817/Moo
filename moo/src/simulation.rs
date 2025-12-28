use crate::core::state::PhaseSpace;
use crate::platform::compute::ComputeEngine;

pub struct Simulation {
    pub state: PhaseSpace,
    pub compute: ComputeEngine,
    pub n_particles: u32,
    pub running: bool,
}

impl Simulation {
    pub async fn new(device: &wgpu::Device, n_particles: u32) -> Self {
        // --- Physics Setup ---
        let dof = n_particles as usize * 3;
        let mut state = PhaseSpace::new(dof); // Fluid particles

        // Initialize Fluid Block
        let spacing = 15.0; // Safer density (h=25.0)
        let cols = 64;
        let start_y = -300.0; // Start lower, effectively -300.0 to +1140.0 range. 
        // 64*15 = 960. -300 + 960 = 660.
        // Center of mass ~ 180. Visible.

        for i in 0..n_particles as usize {
            let col = i % cols;
            let row = i / cols;
            state.q[i * 3] = (col as f64) * spacing - (cols as f64 * spacing / 2.0);
            state.q[i * 3 + 1] = start_y + (row as f64) * spacing;
            state.q[i * 3 + 2] = 0.0;

            state.mass[i * 3] = 1.0;
            state.mass[i * 3 + 1] = 1.0;
            state.mass[i * 3 + 2] = 1.0;

            state.radius[i] = spacing / 2.0;
        }

        // --- GPGPU Setup ---
        let compute = ComputeEngine::new(device, n_particles).await;

        Self {
            state,
            compute,
            n_particles,
            running: false, // Debug: Start paused
        }
    }

    pub fn reset(&mut self, queue: &wgpu::Queue) {
        // Re-initialize state logic here (duplicated from new for now)
        let spacing = 10.0;
        let cols = 10;
        let start_y = 100.0;

        for i in 0..self.n_particles as usize {
            let col = i % cols;
            let row = i / cols;
            self.state.q[i * 3] = (col as f64) * spacing - (cols as f64 * spacing / 2.0);
            self.state.q[i * 3 + 1] = start_y + (row as f64) * spacing;
            self.state.q[i * 3 + 2] = 0.0;

            // Velocity Reset
            self.state.v[i * 3] = 0.0;
            self.state.v[i * 3 + 1] = 0.0;
            self.state.v[i * 3 + 2] = 0.0;
        }

        // Upload reset state
        self.compute
            .write_state(queue, &self.state.q, &self.state.v, &self.state.mass);
    }

    pub fn step(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.running {
            // GPU Physics Step
            // Run multiple substeps for stability
            for _ in 0..10 {
                self.compute.step(device, queue);
            }
        }
    }

    pub fn interact(&mut self, _queue: &wgpu::Queue, _mouse_pos: [f32; 2], _is_pressed: bool) {
        // Handled directly by compute.write_params in the specific app implementation (Khe).
        // This is kept here for future abstraction if needed.
    }

    pub async fn init_headless() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Headless Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                ..Default::default()
            })
            .await
            .expect("Failed to create device")
    }
}
